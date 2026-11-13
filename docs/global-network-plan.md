# Global network plan

## Product requirement

Phones and computers must be able to join from ordinary internet connections without sharing a Wi-Fi network or configuring port forwarding. Local mDNS is a convenience for same-LAN development only. A shared internet connection does not guarantee that another peer can initiate an inbound connection: mobile and residential networks commonly use carrier-grade or router NAT.

The first useful product means one signed, versioned model family that many devices can use. Each device independently runs an inference request suited to its available resources. It does not mean splitting every inference across many phones.

## Connectivity architecture

1. **Outbound bootstrap:** A node opens outbound encrypted connections to several independently operated, publicly reachable bootstrap peers. These seed network discovery and routing; they do not decide which tasks run or control model results. Bootstrap addresses and keys must be replaceable without an app release. Prefer QUIC/UDP where available; provide TCP and, for networks that only allow web egress, a TLS/WebSocket relay endpoint on port 443. No client can connect when a network blocks every supported outbound route.
2. **Peer and capability discovery:** A self-hostable server combines Circuit Relay v2 with a bounded Rendezvous registry for short-lived peer addresses. Android and CLI clients can connect to a configured instance and discover peers across unrelated networks. This is not a Kademlia DHT and does not advertise compute/model capabilities. The server caps registrations per peer and total, bounds their TTL, and caps cached discovery cookies. Publicly reachable server nodes can later maintain Kademlia routing, while phones use a low-resource client role when they are not publicly dialable. Signed provider records should advertise model digest, supported task types, and coarse resource capacity without publishing prompts, personal data, or precise device telemetry.
3. **NAT detection and direct connection:** AutoNAT estimates whether a peer is publicly reachable. When two peers are behind NAT, they first use a relay path to coordinate DCUtR hole punching. The implementation prefers a direct encrypted connection when available.
4. **Relay fallback:** Circuit Relay v2 provides connectivity when direct dialing or hole punching fails. Relay servers accept outbound reservations from phones and enforce per-peer connection, duration, and byte limits. Relay operators can observe peer identities, network addresses, timing, and traffic volume, but end-to-end libp2p encryption protects task contents.
5. **Mobile lifecycle:** Android joins over Wi-Fi or mobile data using outbound connections. Participation remains opt-in and foreground-controlled initially; Android may pause or terminate background work, so the design cannot promise that a phone is always online. Reconnection must handle network changes and sleep/resume.

The network must not depend on one bootstrap provider. The initial test network may use a small, replaceable set of project-operated public nodes, but it must support multiple regions/operators and self-hosted nodes. Losing every bootstrap endpoint can prevent new peers from joining; losing every relay can partition peers behind restrictive NATs. These are availability risks, not authority over compute or governance.

## Task and model routing

After connectivity works, a requester resolves providers for a specific signed model digest and capability, then selects a peer based on resource policy, reachability, latency, and recent reliability. It must not send user prompts to a random discovered peer. Each assignment needs a bounded request, an expiry/cancellation rule, an idempotent task ID, and a result-verification policy. Model artifacts use signed manifests and content-addressed chunks; the DHT stores routing/provider metadata, not large model files.

## Delivery sequence

1. Keep mDNS as a local development path and finish the two-device LAN demonstration.
2. A self-hostable server binary now provides Circuit Relay v2 and bounded Rendezvous discovery over TCP and QUIC, with persistent identity and a Docker image. Android and CLI clients can use a configured server multiaddress, reserve a relay path, publish it to Rendezvous, and refresh discovered peers. The next server work is deployment, health metrics, DHT/bootstrap service, and multiple independently operated endpoints before shipping defaults in Android. See [relay deployment](../deploy/relay/README.md).
3. Validate client relay reservations and discovery on separate home/mobile networks, then add AutoNAT and DCUtR so direct connections can replace relayed traffic when NAT permits. Add a TLS/WebSocket transport path for restrictive networks.
4. Validate devices across separate home/mobile networks, including carrier-grade NAT, relay failure, network switching, and app lifecycle. Record direct-versus-relayed outcomes and bandwidth.
5. Add signed model capability/provider records, task leases, verification, and model distribution. Only then integrate on-device inference.
6. Add independent relay/bootstrap operators, abuse controls, reputation that cannot grant governance power, and failure recovery.

## Scope boundary

The Android/CLI clients can use an operator-supplied Circuit Relay v2/Rendezvous endpoint in addition to direct dialing and same-LAN mDNS. No public server is deployed by the project. DHT/bootstrap service, AutoNAT, DCUtR, capability discovery, and model inference remain unimplemented. Phones can use a third-party/self-hosted public endpoint over mobile data or Wi-Fi when the Wi-Fi-only resource policy is disabled. A code change alone cannot make phones globally reachable: a publicly reachable relay/Rendezvous network must also be deployed and kept available.
