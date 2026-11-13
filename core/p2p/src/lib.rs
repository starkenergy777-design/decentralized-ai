use std::{collections::HashSet, error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    PeerId, StreamProtocol,
    core::transport::ListenerId,
    identify, mdns, noise, ping, relay, rendezvous, request_response,
    request_response::cbor::codec::Codec as CborCodec,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
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
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
    relay_client: relay::client::Behaviour,
    rendezvous: rendezvous::client::Behaviour,
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
    PeerDiscovered(String),
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
const MAX_DISCOVERED_PEER_CONNECTIONS: usize = 8;
const RENDEZVOUS_NAMESPACE: &str = "decentralized-ai";
const RENDEZVOUS_REGISTRATION_TTL_SECONDS: u64 = 30 * 60;
const RENDEZVOUS_PEER_LIMIT: u64 = 64;
const RENDEZVOUS_REFRESH_SECONDS: u64 = 60;
const RENDEZVOUS_REREGISTER_TICKS: u16 = 15;

/// Runs one P2P node until its stop signal arrives or the swarm fails.
pub async fn run_node(
    dial_address: Option<Multiaddr>,
    mut pending_task: Option<TaskRequest>,
    startup: Option<std::sync::mpsc::Sender<NodeInfo>>,
    mut stop: Option<oneshot::Receiver<()>>,
    events: Option<std::sync::mpsc::Sender<NodeEvent>>,
    identity_seed: Option<[u8; 32]>,
    rendezvous_server: Option<Multiaddr>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let identity_lifetime = if identity_seed.is_some() {
        "persistent (Android Keystore-protected seed)"
    } else {
        "process (ephemeral prototype key)"
    };
    let identity = match identity_seed {
        Some(seed) => libp2p::identity::Keypair::ed25519_from_bytes(seed)?,
        None => libp2p::identity::Keypair::generate_ed25519(),
    };
    let peer_id = PeerId::from(identity.public());
    let rendezvous_peer_id = match rendezvous_server.as_ref() {
        Some(address) => Some(
            peer_id_in_multiaddr(address)
                .ok_or("Rendezvous address must include /p2p/<server-peer-id>")?,
        ),
        None => None,
    };
    let identify_config = identify::Config::new("/ai/peer/1".to_owned(), identity.public());
    let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id.clone())?;
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
        .with_dns()?
        .with_relay_client(noise::Config::new, yamux::Config::default)?
        .with_behaviour(|keypair, relay_client| Behaviour {
            identify: identify::Behaviour::new(identify_config),
            mdns: mdns_behaviour,
            ping: ping::Behaviour::default(),
            relay_client,
            rendezvous: rendezvous::client::Behaviour::new(keypair.clone()),
            task: task_behaviour,
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let tcp_address: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    let quic_address: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;
    swarm.listen_on(tcp_address)?;
    swarm.listen_on(quic_address)?;
    let mut relay_listener_id = if let Some(address) = rendezvous_server.as_ref() {
        let listener_id = start_relay_listener(&mut swarm, address);
        swarm.dial(address.clone())?;
        println!("rendezvous_server={address}");
        listener_id
    } else {
        None
    };
    println!("node_id={peer_id}");
    println!("identity_lifetime={identity_lifetime}");
    let mut dialing_discovered_peers = HashSet::new();
    let mut rendezvous_connected = false;
    let mut rendezvous_cookie: Option<rendezvous::Cookie> = None;
    let mut rendezvous_refresh =
        tokio::time::interval(Duration::from_secs(RENDEZVOUS_REFRESH_SECONDS));
    rendezvous_refresh.tick().await;
    let mut rendezvous_ticks = 0_u16;

    if let Some(address) = dial_address {
        swarm.dial(address.clone())?;
        println!("dialing={address}");
    }

    let mut startup_reported = false;
    let expected_task_id = pending_task.as_ref().map(|request| request.id);
    loop {
        let event = tokio::select! {
            _ = async {
                if let Some(stop_receiver) = stop.as_mut() {
                    let _ = stop_receiver.await;
                } else {
                    std::future::pending::<()>().await;
                }
            } => break,
            _ = rendezvous_refresh.tick(), if rendezvous_server.is_some() => {
                if !rendezvous_connected {
                    if let Some(address) = rendezvous_server.as_ref() {
                        if let Err(error) = swarm.dial(address.clone()) {
                            eprintln!("rendezvous_reconnect_failed={error}");
                        }
                    }
                } else {
                    if relay_listener_id.is_none() {
                        if let Some(address) = rendezvous_server.as_ref() {
                            relay_listener_id = start_relay_listener(&mut swarm, address);
                        }
                    }
                    rendezvous_ticks = rendezvous_ticks.wrapping_add(1);
                    if rendezvous_ticks % RENDEZVOUS_REREGISTER_TICKS == 0 {
                        if let Some(rendezvous_peer_id) = rendezvous_peer_id.as_ref() {
                            let registration = swarm.behaviour_mut().rendezvous.register(
                                rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE),
                                rendezvous_peer_id.clone(),
                                Some(RENDEZVOUS_REGISTRATION_TTL_SECONDS),
                            );
                            if let Err(error) = registration {
                                eprintln!("rendezvous_register_failed={error:?}");
                            }
                        }
                    }
                    if let Some(rendezvous_peer_id) = rendezvous_peer_id.as_ref() {
                        swarm.behaviour_mut().rendezvous.discover(
                            Some(rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                            rendezvous_cookie.take(),
                            Some(RENDEZVOUS_PEER_LIMIT),
                            rendezvous_peer_id.clone(),
                        );
                    }
                }
                continue;
            }
            event = swarm.select_next_some() => event,
        };

        match event {
            SwarmEvent::NewListenAddr {
                listener_id,
                address,
            } => {
                println!("listening_on={address}");
                if Some(listener_id) == relay_listener_id
                    && address
                        .iter()
                        .any(|protocol| matches!(protocol, libp2p::multiaddr::Protocol::P2pCircuit))
                {
                    swarm.add_external_address(address.clone());
                    if let Some(rendezvous_peer_id) = rendezvous_peer_id.as_ref() {
                        let registration = swarm.behaviour_mut().rendezvous.register(
                            rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE),
                            rendezvous_peer_id.clone(),
                            Some(RENDEZVOUS_REGISTRATION_TTL_SECONDS),
                        );
                        if let Err(error) = registration {
                            eprintln!("rendezvous_register_failed={error:?}");
                        }
                    }
                }
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
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } if Some(listener_id) == relay_listener_id => {
                println!("relay_listener_closed reason={reason:?}");
                for address in addresses {
                    swarm.remove_external_address(&address);
                }
                relay_listener_id = None;
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                dialing_discovered_peers.remove(&peer_id);
                println!("connected={peer_id}");
                report_event(&events, NodeEvent::PeerConnected(peer_id.to_string()));
                if rendezvous_peer_id.as_ref() == Some(&peer_id) {
                    rendezvous_connected = true;
                    swarm.behaviour_mut().rendezvous.discover(
                        Some(rendezvous::Namespace::from_static(RENDEZVOUS_NAMESPACE)),
                        rendezvous_cookie.take(),
                        Some(RENDEZVOUS_PEER_LIMIT),
                        peer_id.clone(),
                    );
                }
                if endpoint.is_dialer() && rendezvous_peer_id.as_ref() != Some(&peer_id) {
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
                if rendezvous_peer_id.as_ref() == Some(&peer_id) {
                    rendezvous_connected = false;
                }
                report_event(&events, NodeEvent::PeerDisconnected(peer_id.to_string()));
            }
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id.as_ref() {
                    dialing_discovered_peers.remove(peer_id);
                }
                eprintln!("connection_failed peer={peer_id:?} error={error}");
                report_event(
                    &events,
                    NodeEvent::ConnectionFailed(peer_id.map(|peer| peer.to_string())),
                );
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(peers))) => {
                for (peer_id, address) in peers {
                    report_event(&events, NodeEvent::PeerDiscovered(peer_id.to_string()));
                    if swarm.is_connected(&peer_id)
                        || dialing_discovered_peers.contains(&peer_id)
                        || swarm.connected_peers().count() + dialing_discovered_peers.len()
                            >= MAX_DISCOVERED_PEER_CONNECTIONS
                    {
                        continue;
                    }

                    println!("peer_discovered={peer_id} address={address}; connecting");
                    dialing_discovered_peers.insert(peer_id.clone());
                    if let Err(error) = swarm.dial(peer_id.clone()) {
                        dialing_discovered_peers.remove(&peer_id);
                        eprintln!("discovered_peer_dial_failed peer={peer_id} error={error}");
                        report_event(
                            &events,
                            NodeEvent::ConnectionFailed(Some(peer_id.to_string())),
                        );
                    }
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Expired(peers))) => {
                for (peer_id, address) in peers {
                    println!("peer_discovery_expired={peer_id} address={address}");
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Rendezvous(event)) => match event {
                rendezvous::client::Event::Discovered {
                    registrations,
                    cookie,
                    ..
                } => {
                    rendezvous_cookie = Some(cookie);
                    for registration in registrations {
                        let discovered_peer = registration.record.peer_id();
                        if discovered_peer == peer_id {
                            continue;
                        }
                        for address in registration.record.addresses() {
                            let peer_suffix = Multiaddr::empty()
                                .with(libp2p::multiaddr::Protocol::P2p(discovered_peer.clone()));
                            let address = if address.ends_with(&peer_suffix) {
                                address.clone()
                            } else {
                                address
                                    .clone()
                                    .with(libp2p::multiaddr::Protocol::P2p(discovered_peer.clone()))
                            };
                            swarm.add_peer_address(discovered_peer.clone(), address);
                        }
                        report_event(
                            &events,
                            NodeEvent::PeerDiscovered(discovered_peer.to_string()),
                        );
                        if swarm.is_connected(&discovered_peer)
                            || dialing_discovered_peers.contains(&discovered_peer)
                            || swarm.connected_peers().count() + dialing_discovered_peers.len()
                                >= MAX_DISCOVERED_PEER_CONNECTIONS
                        {
                            continue;
                        }

                        println!("rendezvous_peer_discovered={discovered_peer}; connecting");
                        dialing_discovered_peers.insert(discovered_peer);
                        if let Err(error) = swarm.dial(discovered_peer) {
                            dialing_discovered_peers.remove(&discovered_peer);
                            eprintln!(
                                "rendezvous_peer_dial_failed peer={discovered_peer} error={error}"
                            );
                            report_event(
                                &events,
                                NodeEvent::ConnectionFailed(Some(discovered_peer.to_string())),
                            );
                        }
                    }
                }
                rendezvous::client::Event::Registered {
                    rendezvous_node, ..
                } => {
                    println!("rendezvous_registered={rendezvous_node}");
                }
                rendezvous::client::Event::RegisterFailed {
                    rendezvous_node,
                    error,
                    ..
                } => {
                    eprintln!("rendezvous_register_failed node={rendezvous_node} error={error:?}");
                }
                rendezvous::client::Event::DiscoverFailed {
                    rendezvous_node,
                    error,
                    ..
                } => {
                    eprintln!("rendezvous_discover_failed node={rendezvous_node} error={error:?}");
                }
                rendezvous::client::Event::Expired { peer } => {
                    println!("rendezvous_registration_expired={peer}");
                }
            },
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

fn start_relay_listener(swarm: &mut Swarm<Behaviour>, address: &Multiaddr) -> Option<ListenerId> {
    let circuit_address = address
        .clone()
        .with(libp2p::multiaddr::Protocol::P2pCircuit);
    match swarm.listen_on(circuit_address) {
        Ok(listener_id) => Some(listener_id),
        Err(error) => {
            eprintln!("relay_listener_start_failed={error}");
            None
        }
    }
}

fn peer_id_in_multiaddr(address: &Multiaddr) -> Option<PeerId> {
    address.iter().find_map(|protocol| match protocol {
        libp2p::multiaddr::Protocol::P2p(peer_id) => Some(peer_id),
        _ => None,
    })
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

