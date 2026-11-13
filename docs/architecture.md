# Architecture (draft)

## Scope

This repository is an experimental implementation plan and prototype. The network should not depend on a single permanent coordinator. Bootstrap peers may help new nodes find the network, but must not control membership, task execution, or governance.

## Components

| Component | Responsibility | Initial status |
| --- | --- | --- |
| Rust core | Node identity, P2P transport, protocol handling | Prototype |
| Android app (Kotlin) | User interface, permissions, local resource controls, bounded local demo worker | Local demo; P2P bridge planned |
| Desktop node (Rust) | P2P participation and optional compute | Planned |
| Protocol schemas | Versioned task contract shared across clients | Task v1 drafted and mirrored in Rust/Kotlin |
| Compute | Sandboxed, bounded, verifiable task execution | Planned |
| Storage | Content-addressed objects and metadata | Planned |
| Governance | Proposal/vote protocol; test-only until identity is solved | Planned |
| AI | Inference, evaluation, and later training | Planned |
| Updates | Signed releases, staged rollout, rollback | Planned |

## Identity and transport

Each node gets a cryptographic network identity. A network identity identifies a peer; it does not grant governance rights. The initial prototype listens on TCP with Noise/Yamux and QUIC (whose secure channel is provided by QUIC/TLS). The current process key is ephemeral and changes on restart; persistent, protected key storage is a follow-up task. Discovery and NAT traversal are separate milestones. Bootstrap addresses are replaceable and are not trusted as authorities.

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

The current `/ai/task/1` Rust prototype accepts a versioned typed request with fixed-size numeric fields and caps CBOR requests and responses at 256 bytes; it never accepts executable code. Rust rejects unknown fields and validates the task ID, version, operation, and overflow before returning a result. Android applies the same logical contract to a local WorkManager demo, but Android and Rust do not communicate yet. Android P2P integration remains open work.

## Compute and AI

Task execution must be opt-in, resource-limited, cancellable, and isolated from node keys. Results need independent verification. Model updates are untrusted input and require evaluation against fixed baselines and poisoning checks. Start with a small, license-compatible model; training a large model from scratch is outside the first milestone.

## Governance boundary

Desktop nodes and unverified phones can contribute compute but cannot vote. A simulated governance flow may be built for protocol testing, but must never grant real authority. Biometric material must remain local; remote peers receive only a narrowly scoped proof, if a future privacy/security review supports one.

## Change process

AI-generated code is a proposal. It must pass static analysis, dependency checks, isolated tests without network access or secrets, independent review, and governance approval for protected components. The AI cannot change its own safety rules, attestation policy, governance thresholds, or resource limits.
