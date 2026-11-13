# Decentralized AI

A research prototype for a decentralized peer-to-peer AI network. The project is in its initial architecture and networking phase; it is not production-ready and does not implement voting, biometrics, hardware uniqueness, model training, or self-modifying updates.

## Goals

- Peer-to-peer communication without a mandatory central coordinator.
- Shared Rust networking core, with Android and desktop clients.
- Optional compute contributions controlled by each device owner.
- Governance eligibility designed separately from network identity.
- Safety boundaries that keep generated changes out of the live network until reviewed.

## Current status

The prototype has a Rust/libp2p node and an Android foreground P2P control. Android starts and stops the shared Rust node through JNI, displays its peer ID and a dialable local listen address when available, and can initiate a direct connection to one user-supplied peer multiaddress. Android's Ed25519 seed is stored encrypted with AES-GCM using an AES key managed by Android Keystore, so its peer ID persists across ordinary app and device restarts while that key remains available. The seed is briefly present in app and Rust process memory while the node runs. Desktop/CLI nodes still use ephemeral identities. Nodes can discover peers on the same local network with mDNS. When configured with a public relay/Rendezvous server multiaddress, Android and CLI nodes make outbound connections, reserve a Circuit Relay v2 path, register the address, and periodically discover peers; automatic discovery dialing is capped at eight peers. Mobile data works when the Android Wi-Fi-only policy is disabled and an endpoint is available. The project has not deployed its own public server. A sender can optionally submit the bounded `20 + 22` demo task to a manually addressed or discovered peer. The node stops when the app leaves the foreground or the user opts out. It supports one versioned, bounded demonstration task: a peer can request checked integer addition and receive either a result or a typed error. The CBOR task codec caps requests and responses at 256 bytes. Android's local WorkManager demo also calls the shared Rust task protocol crate through JNI. The self-hostable server binary provides Circuit Relay v2 and bounded Rendezvous peer discovery over TCP and QUIC, with persistent identity and a Docker image. Public DHT/bootstrap service, AutoNAT, DCUtR hole punching, global task routing, model inference, distributed training, and background Android participation are not implemented. The target is to join over ordinary Wi-Fi or mobile data through outbound connections, with direct links when possible and encrypted relays when NAT prevents them. Hardware uniqueness and biometric liveness are explicitly unresolved research questions. Any governance implementation in the prototype must remain simulated and grant no real voting power.

## Run the local two-node demonstration

Start one node with `cargo run -p decentralized-ai-p2p`. Copy its printed TCP `listening_on` address. In a second terminal, run `cargo run -p decentralized-ai-p2p -- <TCP_MULTIADDR> --add 20 22`. The second node should print the returned result. This prototype uses ephemeral node identities, so the peer ID changes after restart.

See [architecture](docs/architecture.md), [global network plan](docs/global-network-plan.md), [relay deployment](deploy/relay/README.md), [threat model](docs/threat-model.md), and [acceptance criteria](docs/acceptance-criteria.md).
