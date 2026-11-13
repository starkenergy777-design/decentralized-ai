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

## Later milestones

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
