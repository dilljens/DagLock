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
RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/daglock-indexer /usr/local/bin/
EXPOSE 8443
ENV RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD curl -sf http://localhost:8443/v1/health || exit 1
ENTRYPOINT ["daglock-indexer"]
CMD ["--host", "0.0.0.0", "--port", "8443"]
