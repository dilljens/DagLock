#!/usr/bin/env bash
# DagLock Mainnet Deployment Script
set -euo pipefail

echo "=== DagLock Mainnet Deployment ==="

# Check requirements
command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "curl required"; exit 1; }

# Configuration
export DAGLOCK_MESSAGE_KEY=${DAGLOCK_MESSAGE_KEY:?Must set DAGLOCK_MESSAGE_KEY (64 hex chars)}
IMAGE="daglock/indexer:mainnet-$(date +%Y%m%d)"
DB_DIR=${DAGLOCK_DATA_DIR:-/data/daglock}
PORT=${PORT:-8443}
CORS_ORIGIN=${CORS_ORIGIN:-https://daglock.io}
NETWORK=mainnet

mkdir -p "$DB_DIR"

echo "Building Docker image..."
docker build --load -t "$IMAGE" -t "daglock/indexer:latest" .

echo "Running indexer (mainnet)..."
docker run -d --restart=unless-stopped     --name daglock-indexer     -p "$PORT":8443     -v "$DB_DIR":/data     -e DAGLOCK_MESSAGE_KEY     -e RUST_LOG=info     "$IMAGE"     --database-url sqlite:/data/daglock.db     --network "$NETWORK"     --cors-origin "$CORS_ORIGIN"     --allow-mainnet

echo "Waiting for startup..."
sleep 5

if curl -sf http://localhost:$PORT/v1/health >/dev/null 2>&1; then
    echo "Indexer running on port $PORT"
    echo "API: http://localhost:$PORT/v1"
else
    echo "ERROR: Indexer failed to start. Check logs:"
    docker logs daglock-indexer --tail 20
    exit 1
fi
