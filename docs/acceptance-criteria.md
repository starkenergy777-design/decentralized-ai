# Acceptance criteria

## Milestone 1 — architecture and P2P prototype

- [x] Versioned Rust workspace and documented module boundaries.
- [ ] Two nodes create distinct stable identities.
- [x] Nodes establish an authenticated encrypted libp2p connection.
- [x] A bounded ping/identify exchange completes between two local processes.
- [x] One peer can request a bounded checked-add task and receive its result from another peer.
- [x] Integer overflow returns no result instead of wrapping.
- [ ] Rust and Android task v1 implementations interoperate against the shared CBOR schema, including error cases.
- [ ] Malformed and oversized messages are rejected before application processing.
- [x] CI compiles the Rust workspace and runs its automated checks.
- [x] Documentation clearly marks governance, attestation, biometrics, AI training, and self-improvement as unimplemented.

## Global connectivity milestones

- [ ] Several replaceable, independently operated public bootstrap peers seed the global network; no single endpoint controls membership or tasks.
- [ ] A self-hostable relay binary uses persistent identity and bounded reservation/circuit capacity, duration, bytes, and request rates.
- [ ] Nodes can use a TLS/WebSocket relay on port 443 when a network blocks the default QUIC/UDP and TCP paths.
- [ ] Android and desktop nodes discover one another over unrelated home/mobile networks without shared Wi-Fi or manual port forwarding.
- [ ] AutoNAT identifies public versus NAT-bound nodes; Circuit Relay v2 reservations have connection, duration, and byte limits.
- [ ] DCUtR establishes direct connections when possible and transparently retains end-to-end encrypted relay fallback when hole punching fails.
- [ ] Peer connectivity survives Wi-Fi/mobile switching and foreground resume with clear user-visible state and bounded reconnection.
- [ ] Relays cannot read task contents; provider metadata is signed, minimal, and resilient to stale or malicious records.

## Other later milestones

- [ ] Android peers discover/connect on a shared LAN via mDNS; verify multicast-lock lifecycle, peer cap, and task exchange on real devices. This is a local development check only, not global connectivity.
- [ ] Android's Peer ID remains stable after app/device restart using a Keystore-managed encryption key, and changes only after identity storage is lost or reset.
- [x] Android app builds a debug APK in CI.
- [ ] Android's local demo invokes the shared Rust task protocol crate through JNI on supported ABIs.
- [x] Resource preference screen defaults compute opt-in to off and persists the user's selected budgets locally.
- [ ] Local WorkManager demo obeys opt-in, charging/unmetered-network constraints, low-battery restrictions, and cancellation; it does not yet process P2P tasks.
- [ ] Background compute enforces the selected limits, responds to thermal/battery/network constraints, and can be stopped immediately.
- [ ] Task sandboxing, cancellation, resource quotas, and independent result verification.
- [ ] Android and desktop interoperation, reconnect, discovery, and NAT scenarios.
- [ ] Attestation and uniqueness research reviewed against device/vendor limitations.
- [ ] Biometric prototype assessed for consent, local-only processing, spoof resistance, and measured error rates.
- [ ] Governance remains simulation-only until identity and consensus security criteria are satisfied.
- [ ] Model updates have reproducible evaluation, poisoning defenses, and rollback.
- [ ] Generated code is tested without network access or secrets and requires approval before release.

No item is complete until demonstrated in a reproducible environment and recorded with evidence.
