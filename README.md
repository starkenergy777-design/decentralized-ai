# Decentralized AI

A research prototype for a decentralized peer-to-peer AI network. The project is in its initial architecture and networking phase; it is not production-ready and does not implement voting, biometrics, hardware uniqueness, model training, or self-modifying updates.

## Goals

- Peer-to-peer communication without a mandatory central coordinator.
- Shared Rust networking core, with Android and desktop clients.
- Optional compute contributions controlled by each device owner.
- Governance eligibility designed separately from network identity.
- Safety boundaries that keep generated changes out of the live network until reviewed.

## Current status

The first milestone is a small Rust/libp2p connectivity prototype and a documented threat model. The node now supports one bounded demonstration task: a peer can request integer addition and receive the checked result. It does not execute peer-supplied code. Hardware uniqueness and biometric liveness are explicitly unresolved research questions. Any governance implementation in the prototype must remain simulated and grant no real voting power.

## Run the local two-node demonstration

Start one node with `cargo run -p decentralized-ai-p2p`. Copy its printed TCP `listening_on` address. In a second terminal, run `cargo run -p decentralized-ai-p2p -- <TCP_MULTIADDR> --add 20 22`. The second node should print the returned result. This prototype uses ephemeral node identities, so the peer ID changes after restart.

See [architecture](docs/architecture.md), [threat model](docs/threat-model.md), and [acceptance criteria](docs/acceptance-criteria.md).
