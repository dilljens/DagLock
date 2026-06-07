FROM rust:1.91-slim-bookworm AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY contracts contracts/
COPY indexer indexer/
COPY wasm-sdk wasm-sdk/
COPY cli cli/
RUN cargo build --release -p daglock-indexer && cp target/release/daglock-indexer /usr/local/bin/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3 curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/daglock-indexer /usr/local/bin/

# Create non-root user
RUN groupadd -r daglock && useradd -r -g daglock -m -d /home/daglock daglock
RUN mkdir -p /data && chown daglock:daglock /data

USER daglock
WORKDIR /home/daglock

EXPOSE 8443
ENV RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 CMD curl -sf http://localhost:8443/v1/health || exit 1
# Railway overrides CMD with its own start command (no ENTRYPOINT).
CMD ["daglock-indexer", "--host", "0.0.0.0", "--port", "8443", "--database-url", "sqlite:/data/daglock.db"]
