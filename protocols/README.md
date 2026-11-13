# Protocols

Protocol IDs and schemas are versioned here. The node prototype uses libp2p Identify (`/ai/peer/1`), Ping, and a CBOR request-response protocol (`/ai/task/1`). The current request contains only a numeric task ID and two signed 64-bit integers; the only operation is checked addition. This keeps the demo bounded and avoids executing peer-supplied code. Model, training, governance, attestation, and update messages are not implemented yet.

Future schemas must include a version, bounded payload sizes, validation rules, and compatibility behavior. Never serialize secrets or raw biometric samples into a network message.
