use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, identify, noise, ping, request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    task: request_response::cbor::Behaviour<TaskRequest, TaskResponse>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
struct TaskRequest {
    id: u64,
    left: i64,
    right: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct TaskResponse {
    id: u64,
    result: Option<i64>,
}

fn execute_task(request: TaskRequest) -> TaskResponse {
    TaskResponse {
        id: request.id,
        result: request.left.checked_add(request.right),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let identity = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(identity.public());
    let identify_config = identify::Config::new("/ai/peer/1".to_owned(), identity.public());
    let task_behaviour = request_response::cbor::Behaviour::new(
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

    let mut args = std::env::args().skip(1);
    let dial_address = args.next();
    let mut pending_task = match args.next().as_deref() {
        Some("--add") => {
            let left = args
                .next()
                .ok_or("--add requires two integer operands")?
                .parse()?;
            let right = args
                .next()
                .ok_or("--add requires two integer operands")?
                .parse()?;
            Some(TaskRequest { id: 1, left, right })
        }
        Some(other) => return Err(format!("unknown option: {other}").into()),
        None => None,
    };

    if pending_task.is_some() && dial_address.is_none() {
        return Err("a peer multiaddress is required when using --add".into());
    }

    if let Some(address) = dial_address {
        let address: Multiaddr = address.parse()?;
        swarm.dial(address.clone())?;
        println!("dialing={address}");
    }
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("listening_on={address}");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("connected={peer_id}");
                if let Some(request) = pending_task.take() {
                    let request_id = swarm.behaviour_mut().task.send_request(&peer_id, request);
                    println!("task_request_sent={request_id:?} peer={peer_id}");
                }
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("disconnected={peer_id} cause={cause:?}");
            }
            SwarmEvent::Behaviour(BehaviourEvent::Task(event)) => match event {
                request_response::Event::Message { peer, message, .. } => match message {
                    request_response::Message::Request {
                        request, channel, ..
                    } => {
                        let response = execute_task(request);
                        println!("task_received id={} peer={peer}", request.id);
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
                    } => println!(
                        "task_response request={request_id:?} id={} result={:?}",
                        response.id, response.result
                    ),
                },
                request_response::Event::OutboundFailure {
                    peer,
                    request_id,
                    error,
                    ..
                } => eprintln!(
                    "task_request_failed request={request_id:?} peer={peer} error={error}"
                ),
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
}

#[cfg(test)]
mod tests {
    use super::{TaskRequest, execute_task};

    #[test]
    fn addition_returns_checked_result() {
        let response = execute_task(TaskRequest {
            id: 7,
            left: 20,
            right: 22,
        });

        assert_eq!(response.id, 7);
        assert_eq!(response.result, Some(42));
    }

    #[test]
    fn addition_overflow_is_rejected() {
        let response = execute_task(TaskRequest {
            id: 8,
            left: i64::MAX,
            right: 1,
        });

        assert_eq!(response.id, 8);
        assert_eq!(response.result, None);
    }
}
