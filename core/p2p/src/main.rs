use std::{error::Error, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, PeerId, identify, noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};

#[derive(NetworkBehaviour)]
struct NodeBehaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let identity = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(identity.public());
    let identify_config = identify::Config::new("/ai/peer/1".to_owned(), identity.public());

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| NodeBehaviour {
            identify: identify::Behaviour::new(identify_config),
            ping: ping::Behaviour::default(),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    let tcp_address: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    let quic_address: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse()?;
    swarm.listen_on(tcp_address)?;
    swarm.listen_on(quic_address)?;
    println!("node_id={peer_id}");
    println!("identity_lifetime=process (ephemeral prototype key)");

    if let Some(address) = std::env::args().nth(1) {
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
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("disconnected={peer_id} cause={cause:?}");
            }
            SwarmEvent::Behaviour(event) => println!("protocol_event={event:?}"),
            _ => {}
        }
    }
}
