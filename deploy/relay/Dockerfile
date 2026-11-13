FROM rust:1.88-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p decentralized-ai-p2p --bin decentralized-ai-relay

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends libgcc-s1 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data \
    && chown 10001:10001 /data
COPY --from=builder /src/target/release/decentralized-ai-relay /usr/local/bin/decentralized-ai-relay
USER 10001:10001
ENV DAI_RELAY_KEY_FILE=/data/relay-key.pb
VOLUME ["/data"]
EXPOSE 4001/tcp 4001/udp
ENTRYPOINT ["/usr/local/bin/decentralized-ai-relay"]
