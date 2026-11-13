# Global network plan

## Product requirement

Phones and computers must be able to join from ordinary internet connections without sharing a Wi-Fi network or configuring port forwarding. Local mDNS is a convenience for same-LAN development only. A shared internet connection does not guarantee that another peer can initiate an inbound connection: mobile and residential networks commonly use carrier-grade or router NAT.

The first useful product means one signed, versioned model family that many devices can use. Each device independently runs an inference request suited to its available resources. It does not mean splitting every inference across many phones.

## Connectivity architecture

1. **Outbound bootstrap:** A node opens outbound encrypted connections to several independently operated, publicly reachable bootstrap peers. These seed network discovery and routing; they do not decide which tasks run or control model results. Bootstrap addresses and keys must be replaceable without an app release.
2. **Peer and capability discovery:** Publicly reachable server nodes maintain Kademlia routing. Phones use a low-resource client role when they are not publicly dialable. Later, signed provider records advertise model digest, supported task types, and coarse resource capacity; they must not publish prompts, personal data, or precise device telemetry.
3. **NAT detection and direct connection:** AutoNAT estimates whether a peer is publicly reachable. When two peers are behind NAT, they first use a relay path to coordinate DCUtR hole punching. The implementation prefers a direct encrypted connection when available.
4. **Relay fallback:** Circuit Relay v2 provides connectivity when direct dialing or hole punching fails. Relay servers accept outbound reservations from phones and enforce per-peer connection, duration, and byte limits. Relay operators can observe peer identities, network addresses, timing, and traffic volume, but end-to-end libp2p encryption protects task contents.
5. **Mobile lifecycle:** Android joins over Wi-Fi or mobile data using outbound connections. Participation remains opt-in and foreground-controlled initially; Android may pause or terminate background work, so the design cannot promise that a phone is always online. Reconnection must handle network changes and sleep/resume.

The network must not depend on one bootstrap provider. The initial test network may use a small, replaceable set of project-operated public nodes, but it must support multiple regions/operators and self-hosted nodes. Losing every bootstrap endpoint can prevent new peers from joining; losing every relay can partition peers behind restrictive NATs. These are availability risks, not authority over compute or governance.

## Task and model routing

After connectivity works, a requester resolves providers for a specific signed model digest and capability, then selects a peer based on resource policy, reachability, latency, and recent reliability. It must not send user prompts to a random discovered peer. Each assignment needs a bounded request, an expiry/cancellation rule, an idempotent task ID, and a result-verification policy. Model artifacts use signed manifests and content-addressed chunks; the DHT stores routing/provider metadata, not large model files.

## Delivery sequence

1. Keep mDNS as a local development path and finish the two-device LAN demonstration.
2. Build a public bootstrap/relay node mode with persistent identity, explicit resource limits, health reporting, and deployable container configuration. Run multiple replaceable instances before shipping a default endpoint in the Android app.
3. Add Android/desktop bootstrap connection, Kademlia discovery, AutoNAT, Circuit Relay v2 client reservations, and DCUtR, retaining relay fallback.
4. Validate devices across separate home/mobile networks, including carrier-grade NAT, relay failure, network switching, and app lifecycle. Record direct-versus-relayed outcomes and bandwidth.
5. Add signed model capability/provider records, task leases, verification, and model distribution. Only then integrate on-device inference.
6. Add independent relay/bootstrap operators, abuse controls, reputation that cannot grant governance power, and failure recovery.

## Scope boundary

The repository currently implements only direct peer dialing and same-LAN mDNS. It has no public bootstrap service, DHT, AutoNAT, relay reservation, DCUtR, global peer registry, or model inference. A code change alone cannot make phones globally reachable: a publicly reachable bootstrap/relay network must also be deployed and kept available.
