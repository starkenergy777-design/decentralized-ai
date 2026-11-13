use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    num::NonZeroU32,
    path::Path,
    time::Duration,
};

use futures::StreamExt;
use libp2p::{
    identify, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId,
};

#[derive(NetworkBehaviour)]
struct RelayServerBehaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    relay: relay::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let key_path = std::env::var_os("DAI_RELAY_KEY_FILE")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "./data/relay-key.pb".into());
    let identity = load_or_create_identity(&key_path)?;
    let peer_id = PeerId::from(identity.public());
    let identify_config = identify::Config::new(
        "/decentralized-ai/relay/1".to_owned(),
        identity.public(),
    );
    let relay_config = bounded_relay_config();
    let relay_behaviour = relay::Behaviour::new(peer_id.clone(), relay_config);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(identity)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|_| RelayServerBehaviour {
            identify: identify::Behaviour::new(identify_config),
            ping: ping::Behaviour::default(),
            relay: relay_behaviour,
        })?
        .build();

    let tcp_address = address_from_env("DAI_RELAY_TCP_LISTEN", "/ip4/0.0.0.0/tcp/4001")?;
    let quic_address =
        address_from_env("DAI_RELAY_QUIC_LISTEN", "/ip4/0.0.0.0/udp/4001/quic-v1")?;
    swarm.listen_on(tcp_address)?;
    swarm.listen_on(quic_address)?;

    for address in public_addresses_from_env()? {
        println!("relay_public_address={address}/p2p/{peer_id}");
        swarm.add_external_address(address);
    }

    println!("relay_peer_id={peer_id}");
    println!("relay_key_file={}", key_path.display());
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("relay_listening_on={address}/p2p/{peer_id}");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("relay_peer_connected={peer_id}");
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                println!("relay_peer_disconnected={peer_id} cause={cause:?}");
            }
            SwarmEvent::Behaviour(event) => println!("relay_event={event:?}"),
            _ => {}
        }
    }
}

fn bounded_relay_config() -> relay::Config {
    let minute = Duration::from_secs(60);
    relay::Config {
        max_reservations: 256,
        max_reservations_per_peer: 2,
        reservation_duration: Duration::from_secs(60 * 60),
        reservation_rate_limiters: Vec::new(),
        max_circuits: 128,
        max_circuits_per_peer: 4,
        max_circuit_duration: Duration::from_secs(120),
        max_circuit_bytes: 8 * 1024 * 1024,
        circuit_src_rate_limiters: Vec::new(),
    }
    .reservation_rate_per_peer(NonZeroU32::new(10).expect("non-zero limit"), minute)
    .reservation_rate_per_ip(NonZeroU32::new(60).expect("non-zero limit"), minute)
    .circuit_src_per_peer(NonZeroU32::new(30).expect("non-zero limit"), minute)
    .circuit_src_per_ip(NonZeroU32::new(120).expect("non-zero limit"), minute)
}

fn address_from_env(name: &str, default: &str) -> Result<Multiaddr, Box<dyn Error + Send + Sync>> {
    let address = std::env::var(name).unwrap_or_else(|_| default.to_owned());
    Ok(address.parse()?)
}

fn public_addresses_from_env() -> Result<Vec<Multiaddr>, Box<dyn Error + Send + Sync>> {
    std::env::var("DAI_RELAY_PUBLIC_ADDRS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|address| !address.is_empty())
        .map(|address| address.parse().map_err(Into::into))
        .collect()
}

fn load_or_create_identity(
    path: &Path,
) -> Result<libp2p::identity::Keypair, Box<dyn Error + Send + Sync>> {
    if path.exists() {
        return read_identity(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let identity = libp2p::identity::Keypair::generate_ed25519();
    let encoded = identity.to_protobuf_encoding()?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(&encoded)?;
            file.sync_all()?;
            Ok(identity)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read_identity(path),
        Err(error) => Err(error.into()),
    }
}

fn read_identity(path: &Path) -> Result<libp2p::identity::Keypair, Box<dyn Error + Send + Sync>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > 1024 {
        return Err("relay identity file is unexpectedly large".into());
    }
    let encoded = fs::read(path)?;
    Ok(libp2p::identity::Keypair::from_protobuf_encoding(&encoded)?)
}
