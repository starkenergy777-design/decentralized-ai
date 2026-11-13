# Decentralized AI

A research prototype for a decentralized peer-to-peer AI network. The project is in its initial architecture and networking phase; it is not production-ready and does not implement voting, biometrics, hardware uniqueness, model training, or self-modifying updates.

## Goals

- Peer-to-peer communication without a mandatory central coordinator.
- Shared Rust networking core, with Android and desktop clients.
- Optional compute contributions controlled by each device owner.
- Governance eligibility designed separately from network identity.
- Safety boundaries that keep generated changes out of the live network until reviewed.

## Current status

The first milestone is a small Rust/libp2p connectivity prototype and a documented threat model. Hardware uniqueness and biometric liveness are explicitly unresolved research questions. Any governance implementation in the prototype must remain simulated and grant no real voting power.

See [architecture](docs/architecture.md), [threat model](docs/threat-model.md), and [acceptance criteria](docs/acceptance-criteria.md).

