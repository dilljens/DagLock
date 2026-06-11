#!/usr/bin/env bash
# DagLock Mainnet Deployment Script
set -euo pipefail

echo "=== DagLock Mainnet Deployment ==="

# Check requirements
command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "curl required"; exit 1; }

# Configuration — required env vars
export DAGLOCK_MESSAGE_KEY=${DAGLOCK_MESSAGE_KEY:?Must set DAGLOCK_MESSAGE_KEY (64 hex chars)}

# Template hashes — run: cargo test -p daglock-contracts -- --nocapture print_template_hashes
DAGLOCK_KAS_TEMPLATE=${DAGLOCK_KAS_TEMPLATE:?Must set DAGLOCK_KAS_TEMPLATE (40 hex chars)}
DAGLOCK_KRC20_TEMPLATE=${DAGLOCK_KRC20_TEMPLATE:?Must set DAGLOCK_KRC20_TEMPLATE (40 hex chars)}
DAGLOCK_VAULT_SOFTLOCK_TEMPLATE=${DAGLOCK_VAULT_SOFTLOCK_TEMPLATE:-}
DAGLOCK_VAULT_MULTISIG_TEMPLATE=${DAGLOCK_VAULT_MULTISIG_TEMPLATE:-}

# wRPC endpoint — connect to a Kaspa node for UTXO scanning
WRPC_URL=${WRPC_URL:-wss://kaspa.infstone.io}

# Treasury public key (64 hex) — used for fee collection and vault sweeps
TREASURY_PUBKEY=${TREASURY_PUBKEY:-}

# Deployment settings
IMAGE="daglock/indexer:mainnet-$(date +%Y%m%d)"
DB_DIR=${DAGLOCK_DATA_DIR:-/data/daglock}
PORT=${PORT:-8443}
CORS_ORIGIN=${CORS_ORIGIN:-https://daglock.io}
NETWORK=mainnet

mkdir -p "$DB_DIR"

echo "Building Docker image..."
docker build --load -t "$IMAGE" -t "daglock/indexer:latest" .

echo "Running indexer (mainnet)..."

# Build argument list
DOCKER_ARGS=(
    -d --restart=unless-stopped
    --name daglock-indexer
    -p "$PORT":8543
    -v "$DB_DIR":/data
    -e DAGLOCK_MESSAGE_KEY
    -e RUST_LOG=info
    "$IMAGE"
    --database-url sqlite:/data/daglock.db
    --network "$NETWORK"
    --cors-origin "$CORS_ORIGIN"
    --allow-mainnet
    --wrpc-url "$WRPC_URL"
    --daglock-kas-template "$DAGLOCK_KAS_TEMPLATE"
    --daglock-krc20-template "$DAGLOCK_KRC20_TEMPLATE"
)

# Add optional flags
if [ -n "$DAGLOCK_VAULT_SOFTLOCK_TEMPLATE" ]; then
    DOCKER_ARGS+=(--daglock-vault-softlock-template "$DAGLOCK_VAULT_SOFTLOCK_TEMPLATE")
fi
if [ -n "$DAGLOCK_VAULT_MULTISIG_TEMPLATE" ]; then
    DOCKER_ARGS+=(--daglock-vault-multisig-template "$DAGLOCK_VAULT_MULTISIG_TEMPLATE")
fi
if [ -n "$TREASURY_PUBKEY" ]; then
    DOCKER_ARGS+=(--treasury-pubkey "$TREASURY_PUBKEY")
fi

docker run "${DOCKER_ARGS[@]}"

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
