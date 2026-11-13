# Architecture (draft)

## Scope

This repository is an experimental implementation plan and prototype. The network should not depend on a single permanent coordinator. Bootstrap peers may help new nodes find the network, but must not control membership, task execution, or governance.

## Components

| Component | Responsibility | Initial status |
| --- | --- | --- |
| Rust core | Node identity, P2P transport, shared task protocol and execution | Prototype |
| Android app (Kotlin) | User interface, opt-in controls, bounded local demo worker, foreground P2P lifecycle | Starts shared Rust/libp2p node through JNI; shows peer ID and listen address |
| Desktop node (Rust) | P2P participation and optional compute | Planned |
| Protocol schemas | Versioned task contract shared across clients | Task v1 drafted and mirrored in Rust/Kotlin |
| Compute | Sandboxed, bounded, verifiable task execution | Planned |
| Storage | Content-addressed objects and metadata | Planned |
| Governance | Proposal/vote protocol; test-only until identity is solved | Planned |
| AI | Inference, evaluation, and later training | Planned |
| Updates | Signed releases, staged rollout, rollback | Planned |

## Identity and transport

Each node gets a cryptographic network identity. A network identity identifies a peer; it does not grant governance rights. The initial prototype listens on TCP with Noise/Yamux and QUIC (whose secure channel is provided by QUIC/TLS). Android stores its random Ed25519 seed encrypted with AES-GCM and an AES key managed by Android Keystore; it supplies the seed to the Rust node at startup, so the Android peer ID persists across ordinary restarts while the Keystore key remains available. The seed is transiently present in app and Rust process memory while starting or running the node; the private key itself is not kept inside Keystore. Desktop/CLI nodes still use ephemeral keys. Nodes use libp2p mDNS to discover and connect to peers on the same LAN, with an eight-peer cap for automatic discovery; mDNS advertisements expose peer IDs to devices on that LAN. This is a local development path, not internet discovery. Before raising Android's target SDK to 37, add the platform's local-network access permission or a system-mediated discovery flow.

The global target must work when peers are on unrelated Wi-Fi or mobile networks. Nodes should make outbound connections to multiple replaceable bootstrap peers; publicly reachable DHT nodes seed routing, while phones use a low-resource client role behind NAT. AutoNAT determines reachability, Circuit Relay v2 provides a reservation and encrypted fallback, and DCUtR attempts a direct path through NAT when possible. A bounded, persistent-identity Circuit Relay v2 server binary now exists, but client reservations, bootstrap/DHT discovery, AutoNAT, and DCUtR are not connected. Bootstrap and relay services help with reachability but must not control task assignment, compute, model results, or governance. See [global network plan](global-network-plan.md) for rollout details and availability assumptions.

Governance identity, if ever activated, must be separate from network identity. No implementation may claim one-device-one-vote until a credible uniqueness mechanism and recovery/revocation rules have been independently reviewed.

## Protocol boundaries

Protocols will be versioned and bounded:

- `/ai/peer/1`: capability exchange and peer metadata.
- `/ai/task/1`: task assignment and cancellation.
- `/ai/result/1`: result submission and verification metadata.
- `/ai/model/1`: model artifact exchange.
- `/ai/train/1`: training update exchange.
- `/ai/governance/1`: proposals and test-only ballots.
- `/ai/attestation/1`: attestation evidence envelopes.
- `/ai/update/1`: signed release metadata.

The current `/ai/task/1` Rust prototype accepts a versioned typed request with fixed-size numeric fields and caps CBOR requests and responses at 256 bytes; it never accepts executable code. Rust rejects unknown fields and validates the task ID, version, operation, and overflow before returning a result. Android starts the same Rust/libp2p node through JNI only after the user opts in and taps the start control. It can directly dial a user-supplied multiaddress or automatically discover and dial up to eight peers on the same LAN using mDNS; a sender can optionally send a single checked `20 + 22` task to the first connected peer. The UI shows connection events and substitutes the active interface's IPv4 address for the wildcard listener address when available. The node stops when the Activity leaves the foreground or participation is disabled. Android's peer ID persists across ordinary restarts, while CLI identity remains ephemeral. Global discovery, NAT traversal, and relay fallback remain unimplemented.

## Compute and AI

Task execution must be opt-in, resource-limited, cancellable, and isolated from node keys. Results need independent verification. Model updates are untrusted input and require evaluation against fixed baselines and poisoning checks. Start with a small, license-compatible model; training a large model from scratch is outside the first milestone.

## Governance boundary

Desktop nodes and unverified phones can contribute compute but cannot vote. A simulated governance flow may be built for protocol testing, but must never grant real authority. Biometric material must remain local; remote peers receive only a narrowly scoped proof, if a future privacy/security review supports one.

## Change process

AI-generated code is a proposal. It must pass static analysis, dependency checks, isolated tests without network access or secrets, independent review, and governance approval for protected components. The AI cannot change its own safety rules, attestation policy, governance thresholds, or resource limits.
