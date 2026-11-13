# Protocols

Protocol IDs and schemas are versioned here. The first node prototype uses libp2p Identify (`/ai/peer/1`) and Ping to verify a QUIC connection. Application task, result, model, training, governance, attestation, and update messages are not implemented yet.

Future schemas must include a version, bounded payload sizes, validation rules, and compatibility behavior. Never serialize secrets or raw biometric samples into a network message.

