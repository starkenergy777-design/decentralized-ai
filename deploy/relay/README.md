# Self-hosted relay node

The `decentralized-ai-relay` binary is the first public-network infrastructure component. It accepts libp2p Circuit Relay v2 reservations over TCP and QUIC. It is not a DHT/bootstrap server, and the Android/desktop clients do not yet request relay reservations.

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

The named volume stores the relay's protobuf-encoded Ed25519 identity. Keep that volume across restarts and protect it as a private key; losing it changes the relay Peer ID. The container runs as an unprivileged user. Startup logs show the Peer ID, listen addresses, and advertised multiaddresses.

## Initial resource limits

- 256 total reservations, at most 2 per peer, renewed for up to 1 hour.
- 128 circuits, at most 4 per peer.
- 120 seconds and 8 MiB per circuit.
- Rate limits on reservation and circuit requests per peer and source IP.

These are prototype defaults. Monitor bandwidth and memory before running a public service. The relay forwards end-to-end encrypted libp2p streams but still sees peer identities, IP addresses, timing, and traffic volume. This initial listener uses TCP port 4001 and QUIC/UDP port 4001; a TLS/WebSocket port 443 fallback has not been implemented.
