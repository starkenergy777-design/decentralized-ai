# Self-hosted relay node

The `decentralized-ai-relay` binary combines a Circuit Relay v2 service with a libp2p Rendezvous peer-discovery service over TCP and QUIC. Rendezvous registrations are limited to 4 per peer and 10,000 total, with bounded TTLs (60 seconds to 30 minutes) and a 4,096-cookie cache. Clients can be configured with this service's multiaddress, but no public instance is deployed by the project. This is not a Kademlia DHT/bootstrap server.

## Build and run with Docker

Run these commands from the repository root. Replace `PUBLIC_IPV4` with the server's stable, publicly reachable IPv4 address and open TCP and UDP port 4001 in the provider firewall.

```sh
docker build -f deploy/relay/Dockerfile -t decentralized-ai-relay .
docker volume create decentralized-ai-relay-data
docker run --name decentralized-ai-relay --restart unless-stopped \
  -p 4001:4001/tcp -p 4001:4001/udp \
  --mount source=decentralized-ai-relay-data,target=/data \
  -e DAI_RELAY_PUBLIC_ADDRS="/ip4/PUBLIC_IPV4/tcp/4001,/ip4/PUBLIC_IPV4/udp/4001/quic-v1" \
  decentralized-ai-relay
```

The named volume stores the relay's protobuf-encoded Ed25519 identity. Keep that volume across restarts and protect it as a private key; losing it changes the relay Peer ID. The container runs as an unprivileged user. Startup logs show the Peer ID, listen addresses, and advertised multiaddresses. A client can use the TCP endpoint as `/ip4/PUBLIC_IPV4/tcp/4001/p2p/RELAY_PEER_ID` or the QUIC endpoint as `/ip4/PUBLIC_IPV4/udp/4001/quic-v1/p2p/RELAY_PEER_ID`.

For the CLI, set `DAI_RENDEZVOUS_ADDR` to that multiaddress before running `cargo run -p decentralized-ai-p2p`. In Android, paste it into the public relay/Rendezvous field. The client can then be reached through the relay and will query this server for other registered peers.

## Initial resource limits

- 256 total reservations, at most 2 per peer, renewed for up to 1 hour.
- 128 circuits, at most 4 per peer.
- 120 seconds and 8 MiB per circuit.
- Rate limits on reservation and circuit requests per peer and source IP.
- 10,000 Rendezvous registrations total, 4 per peer, and at most 4,096 cached discovery cookies.
- Rendezvous registration TTL from 60 seconds to 30 minutes.

These are prototype defaults. Monitor bandwidth and memory before running a public service. The relay forwards end-to-end encrypted libp2p streams but still sees peer identities, IP addresses, timing, and traffic volume. Rendezvous stores peer addresses temporarily to let other peers dial them. This initial listener uses TCP port 4001 and QUIC/UDP port 4001; a TLS/WebSocket port 443 fallback has not been implemented.
