# Decentralized AI

A research prototype for a decentralized peer-to-peer AI network. The project is in its initial architecture and networking phase; it is not production-ready and does not implement voting, biometrics, hardware uniqueness, model training, or self-modifying updates.

## Goals

- Peer-to-peer communication without a mandatory central coordinator.
- Shared Rust networking core, with Android and desktop clients.
- Optional compute contributions controlled by each device owner.
- Governance eligibility designed separately from network identity.
- Safety boundaries that keep generated changes out of the live network until reviewed.

## Current status

The prototype now has a Rust/libp2p node and an Android foreground P2P control. Android starts and stops the shared Rust node through JNI, displays its peer ID and a dialable local listen address when available, and can initiate a direct connection to one user-supplied peer multiaddress. Android's Ed25519 seed is stored encrypted with AES-GCM using an AES key managed by Android Keystore, so its peer ID persists across ordinary app and device restarts while that key remains available. The seed is briefly present in app and Rust process memory while the node runs. Desktop/CLI nodes still use ephemeral identities. Nodes also discover and connect to up to eight peers advertised through libp2p mDNS on the same local network; this does not cross NAT or connect different LANs. A sender can optionally submit the bounded `20 + 22` demo task to a manually addressed or first discovered peer and view the response in the app. The node stops when the app leaves the foreground or the user opts out. It supports one versioned, bounded demonstration task: a peer can request checked integer addition and receive either a result or a typed error. The CBOR task codec caps requests and responses at 256 bytes. Android's local WorkManager demo also calls the shared Rust task protocol crate through JNI. Neither side executes peer-supplied code. NAT traversal, relay, model inference, distributed training, and background Android participation are not implemented. Hardware uniqueness and biometric liveness are explicitly unresolved research questions. Any governance implementation in the prototype must remain simulated and grant no real voting power.

## Run the local two-node demonstration

Start one node with `cargo run -p decentralized-ai-p2p`. Copy its printed TCP `listening_on` address. In a second terminal, run `cargo run -p decentralized-ai-p2p -- <TCP_MULTIADDR> --add 20 22`. The second node should print the returned result. This prototype uses ephemeral node identities, so the peer ID changes after restart.

See [architecture](docs/architecture.md), [threat model](docs/threat-model.md), and [acceptance criteria](docs/acceptance-criteria.md).
