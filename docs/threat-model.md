# Threat model (initial)

## Assets

- Node signing keys and device-local secrets.
- Integrity and availability of the peer network.
- Integrity of task results and model artifacts.
- User privacy, especially biometric and device-attestation data.
- User control of CPU, memory, storage, network, and battery use.
- Integrity of governance and software update decisions.

## Adversaries

- Sybil operators creating many network identities.
- Malicious peers sending malformed or oversized protocol messages.
- Participants returning false compute results or poisoned model updates.
- Compromised endpoints, rooted devices, replayed evidence, and relays that observe metadata.
- Malicious or unavailable bootstrap/DHT/relay operators; DHT eclipse or Sybil attempts; carrier-grade NAT and restrictive firewalls that prevent inbound dialing.
- Supply-chain compromise in dependencies, models, or build/release infrastructure.

## Trust assumptions and unresolved questions

- libp2p secure channels authenticate peer keys but do not prove a peer is one unique human or physical device.
- Local mDNS advertisements reveal an opted-in node's Peer ID and network presence to devices on the same LAN; a discovered peer is not trusted until the libp2p secure connection authenticates it.
- Android hardware attestation depends on platform/vendor trust roots and can be unavailable or revoked.
- A compromised Android device can falsify application-level observations; biometric liveness is not a guarantee of device integrity.
- Universal one-physical-phone-one-vote is unresolved. Do not activate real voting based on app-only checks.
- Bootstrap peers and relays may be unavailable or malicious; the protocol must not treat them as authorities.
- Public bootstrap and relay peers are reachability infrastructure, not trusted task schedulers. A relay can observe Peer IDs, IP addresses, timing, and traffic volume even though end-to-end encryption protects task contents.

## Initial mitigations

- Strict message size limits, validation, timeouts, and version negotiation.
- Separate network identity from any future governance credential.
- On Android, encrypt the persistent Ed25519 seed with AES-GCM under a key managed by Android Keystore; erase transient seed buffers after handing identity to the node.
- Treat an unlocked/rooted device or process-memory compromise as able to expose an active Android node's seed; Keystore encryption protects the stored seed, not its runtime use.
- No transmission or persistence of raw biometric samples.
- Keep governance simulation-only until uniqueness, recovery, privacy, and adversarial testing are reviewed.
- Run untrusted tasks in an OS sandbox without access to keys or host credentials.
- Verify artifacts by content hash and signatures; support rollback of staged updates.
- Make resource sharing opt-in and interruptible.
- Enable Android Wi-Fi multicast reception only while the foreground P2P node is running, and release the multicast lock immediately when it stops.
- Keep multiple independent bootstrap/relay choices, bound relay reservations and bytes, rotate endpoints safely, and fail closed when a signed provider record or task result is invalid.

## Out of scope for the first prototype

Production consensus, real voting eligibility, biometric anti-spoof guarantees, production key recovery, and safe arbitrary code execution are not delivered by the initial P2P prototype.
