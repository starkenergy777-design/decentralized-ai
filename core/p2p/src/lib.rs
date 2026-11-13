use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    PeerId, StreamProtocol, identify, noise, ping, request_response,
    request_response::cbor::codec::Codec as CborCodec,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use tokio::sync::oneshot;

pub use libp2p::Multiaddr;

pub use decentralized_ai_task_protocol::{
    ERROR_OVERFLOW, ERROR_UNSUPPORTED_OPERATION, ERROR_UNSUPPORTED_VERSION, OP_ADD, TASK_VERSION,
    TaskRequest, TaskResponse, execute_task,
};

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    task: request_response::cbor::Behaviour<TaskRequest, TaskResponse>,
}

#[derive(Clone, Debug)]
pub struct NodeInfo {
    pub peer_id: String,
    pub listen_address: String,
}

#[derive(Clone, Debug)]
pub enum NodeEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    ConnectionFailed(Option<String>),
    TaskSent(i64),
    TaskFailed,
    TaskReceived(i64),
    TaskResponse {
        id: i64,
        result: Option<i64>,
        error: Option<u8>,
    },
}

const MAX_TASK_MESSAGE_BYTES: u64 = 256;

/// Runs one P2P node until its stop signal arrives or the swarm fails.
pub async fn run_node(
    dial_address: Option<Multiaddr>,
    mut pending_task: Option<TaskRequest>,
    startup: Option<std::sync::mpsc::Sender<NodeInfo>>,
    mut stop: Option<oneshot::Receiver<()>>,
    events: Option<std::sync::mpsc::Sender<NodeEvent>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let identity = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(identity.public());
    let identify_config = identify::Config::new("/ai/peer/1".to_owned(), identity.public());
    let task_codec = CborCodec::<TaskRequest, TaskResponse>::default()
        .set_request_size_maximum(MAX_TASK_MESSAGE_BYTES)
        .set_response_size_maximum(MAX_TASK_MESSAGE_BYTES);
    let task_behaviour = request_response::cbor::Behaviour::with_codec(
        task_codec,
        [(
            StreamProtocol::new("/ai/task/1"),
            request_response::ProtocolSupport::Full,
        )],
        request_response::Config::default(),
    );

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| Behaviour {
            identify: identify::Behaviour::new(identify_config),
            ping: ping::Behaviour::default(),
            task: task_behaviour,
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let tcp_address: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    let quic_address: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;
    swarm.listen_on(tcp_address)?;
    swarm.listen_on(quic_address)?;
    println!("node_id={peer_id}");
    println!("identity_lifetime=process (ephemeral prototype key)");

    if pending_task.is_some() && dial_address.is_none() {
        return Err("a peer multiaddress is required when using --add".into());
    }

    if let Some(address) = dial_address {
        swarm.dial(address.clone())?;
        println!("dialing={address}");
    }

    let mut startup_reported = false;
    let expected_task_id = pending_task.as_ref().map(|request| request.id);
    loop {
        let event = if let Some(stop_receiver) = stop.as_mut() {
            tokio::select! {
                _ = stop_receiver => break,
                event = swarm.select_next_some() => event,
            }
        } else {
            swarm.select_next_some().await
        };

        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("listening_on={address}");
                if !startup_reported {
                    startup_reported = true;
                    if let Some(startup) = startup.as_ref() {
                        let _ = startup.send(NodeInfo {
                            peer_id: peer_id.to_string(),
                            listen_address: address.to_string(),
                        });
                    }
                }
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                println!("connected={peer_id}");
                report_event(&events, NodeEvent::PeerConnected(peer_id.to_string()));
                if endpoint.is_dialer() {
                    if let Some(request) = pending_task.take() {
                        let task_id = request.id;
                        let request_id = swarm.behaviour_mut().task.send_request(&peer_id, request);
                        println!("task_request_sent={request_id:?} peer={peer_id}");
                        report_event(&events, NodeEvent::TaskSent(task_id));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("disconnected={peer_id} cause={cause:?}");
                report_event(&events, NodeEvent::PeerDisconnected(peer_id.to_string()));
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                eprintln!("connection_failed peer={peer_id:?} error={error}");
                report_event(
                    &events,
                    NodeEvent::ConnectionFailed(peer_id.map(|peer| peer.to_string())),
                );
            }
            SwarmEvent::Behaviour(BehaviourEvent::Task(event)) => match event {
                request_response::Event::Message { peer, message, .. } => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        let request_id = request.id;
                        let response = execute_task(request);
                        println!("task_received id={request_id} peer={peer}");
                        report_event(&events, NodeEvent::TaskReceived(request_id));
                        if swarm
                            .behaviour_mut()
                            .task
                            .send_response(channel, response)
                            .is_err()
                        {
                            eprintln!("task_response_dropped peer={peer}");
                        }
                    }
                    request_response::Message::Response {
                        request_id,
                        response,
                    } => {
                        println!(
                            "task_response request={request_id:?} id={} result={:?} error={:?}",
                            response.id, response.result, response.error
                        );
                        if response.version == TASK_VERSION
                            && Some(response.id) == expected_task_id
                            && response.result.is_some() != response.error.is_some()
                        {
                            report_event(
                                &events,
                                NodeEvent::TaskResponse {
                                    id: response.id,
                                    result: response.result,
                                    error: response.error,
                                },
                            );
                        } else {
                            eprintln!("task_response_invalid request={request_id:?}");
                            report_event(&events, NodeEvent::TaskFailed);
                        }
                    }
                },
                request_response::Event::OutboundFailure {
                    peer,
                    request_id,
                    error,
                    ..
                } => {
                    eprintln!(
                        "task_request_failed request={request_id:?} peer={peer} error={error}"
                    );
                    report_event(&events, NodeEvent::TaskFailed);
                }
                request_response::Event::InboundFailure { peer, error, .. } => {
                    eprintln!("task_request_rejected peer={peer} error={error}");
                }
                request_response::Event::ResponseSent { peer, .. } => {
                    println!("task_response_sent peer={peer}");
                }
            },
            SwarmEvent::Behaviour(event) => println!("protocol_event={event:?}"),
            _ => {}
        }
    }

    Ok(())
}

fn report_event(events: &Option<std::sync::mpsc::Sender<NodeEvent>>, event: NodeEvent) {
    if let Some(events) = events {
        let _ = events.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::{ERROR_OVERFLOW, OP_ADD, TASK_VERSION, TaskRequest, execute_task};

    #[test]
    fn addition_returns_checked_result() {
        let response = execute_task(TaskRequest {
            version: TASK_VERSION,
            id: 7,
            operation: OP_ADD,
            left: 20,
            right: 22,
        });

        assert_eq!(response.version, TASK_VERSION);
        assert_eq!(response.id, 7);
        assert_eq!(response.result, Some(42));
        assert_eq!(response.error, None);
    }

    #[test]
    fn addition_overflow_is_rejected() {
        let response = execute_task(TaskRequest {
            version: TASK_VERSION,
            id: 8,
            operation: OP_ADD,
            left: i64::MAX,
            right: 1,
        });

        assert_eq!(response.id, 8);
        assert_eq!(response.result, None);
        assert_eq!(response.error, Some(ERROR_OVERFLOW));
    }
}
