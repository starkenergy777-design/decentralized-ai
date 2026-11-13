# Desktop node

The first experimental node implementation lives in `core/p2p` and runs as a Rust process. Set `DAI_RENDEZVOUS_ADDR` to a public relay/Rendezvous multiaddress ending in `/p2p/<server-peer-id>` to connect over the internet; without it, the CLI retains direct dialing and same-LAN mDNS only. This folder is reserved for a packaged desktop client, configuration UI, and user-facing resource controls. The command-line prototype currently uses an ephemeral identity and is not suitable for unattended or production operation.
