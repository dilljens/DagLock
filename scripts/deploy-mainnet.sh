#!/usr/bin/env bash
# DagLock Deploy Script (testnet-12 by default, set NETWORK=mainnet for mainnet)
set -euo pipefail

echo "=== DagLock Indexer Deploy ==="

# Check requirements
command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "curl required"; exit 1; }

# Configuration — required env vars (only for mainnet)
DAGLOCK_MESSAGE_KEY=${DAGLOCK_MESSAGE_KEY:-}

# Template hashes — run: cargo test -p daglock-contracts -- --nocapture print_template_hashes
DAGLOCK_KAS_TEMPLATE=${DAGLOCK_KAS_TEMPLATE:-30876e3ea42d0e23bb0980f3fd97ae8807e9c70f}
DAGLOCK_KRC20_TEMPLATE=${DAGLOCK_KRC20_TEMPLATE:-ae0946e4a9bd4a7585e6bf9135de38083cb11c85}
DAGLOCK_REPUTATION_TEMPLATE=${DAGLOCK_REPUTATION_TEMPLATE:-65c54102c64a331414b602760cbd76efac3d69df}
DAGLOCK_VAULT_SOFTLOCK_TEMPLATE=${DAGLOCK_VAULT_SOFTLOCK_TEMPLATE:-ed57b9da957beaac387a0baa9a23c8c54d186964}
DAGLOCK_VAULT_MULTISIG_TEMPLATE=${DAGLOCK_VAULT_MULTISIG_TEMPLATE:-caf0b46ea425159b80af81436fc8f8cfd4e62afa}
DAGLOCK_VAULT_TEMPLATE=${DAGLOCK_VAULT_TEMPLATE:-b338c514b1ef79bf1b0739814bc0d567e8461cfb}

# wRPC endpoint — connect to a Kaspa node for UTXO scanning.
# For mainnet, known working endpoints:
#   wss://troy.kaspa.stream/kaspa/mainnet/wrpc/borsh
#   wss://maxim.kaspa.stream/kaspa/mainnet/wrpc/borsh
# Default: empty = --no-wrpc (MockVerifier). No testnet wRPC endpoint is currently available.
WRPC_URL=${WRPC_URL:-}

# Treasury public key (64 hex) — used for fee collection and vault sweeps.
# Generate a keypair for the DagLock treasury before mainnet launch.
TREASURY_PUBKEY=${TREASURY_PUBKEY:-}

# Deployment settings
NETWORK=${NETWORK:-testnet-12}
IMAGE="daglock/indexer:${NETWORK}-$(date +%Y%m%d)"
DB_DIR=${DAGLOCK_DATA_DIR:-/data/daglock}
PORT=${PORT:-8443}
CORS_ORIGIN=${CORS_ORIGIN:-https://daglock.com}

mkdir -p "$DB_DIR"

echo "Building Docker image..."
docker build --load -t "$IMAGE" -t "daglock/indexer:latest" .

echo "Running indexer (${NETWORK})..."

# Build argument list
DOCKER_ARGS=(
    -d --restart=unless-stopped
    --name daglock-indexer
    -p "$PORT":8543
    -v "$DB_DIR":/data
    -e RUST_LOG=info
    "$IMAGE"
    --database-url sqlite:/data/daglock.db
    --network "$NETWORK"
    --cors-origin "$CORS_ORIGIN"
    --daglock-kas-template "$DAGLOCK_KAS_TEMPLATE"
    --daglock-krc20-template "$DAGLOCK_KRC20_TEMPLATE"
    --daglock-reputation-template "$DAGLOCK_REPUTATION_TEMPLATE"
)

# Mainnet-only flags
if [ "$NETWORK" = "mainnet" ]; then
    DOCKER_ARGS+=(--allow-mainnet)
    if [ -z "$DAGLOCK_MESSAGE_KEY" ]; then
        echo "ERROR: DAGLOCK_MESSAGE_KEY is required for mainnet"
        exit 1
    fi
    DOCKER_ARGS+=(-e DAGLOCK_MESSAGE_KEY)
fi

# wRPC or offline mode
if [ -n "$WRPC_URL" ]; then
    DOCKER_ARGS+=(--wrpc-url "$WRPC_URL")
else
    DOCKER_ARGS+=(--no-wrpc)
    echo "  --no-wrpc (offline mode, no UTXO verification)"
fi

# Add optional template flags
if [ -n "$DAGLOCK_VAULT_SOFTLOCK_TEMPLATE" ]; then
    DOCKER_ARGS+=(--daglock-vault-softlock-template "$DAGLOCK_VAULT_SOFTLOCK_TEMPLATE")
fi
if [ -n "$DAGLOCK_VAULT_MULTISIG_TEMPLATE" ]; then
    DOCKER_ARGS+=(--daglock-vault-multisig-template "$DAGLOCK_VAULT_MULTISIG_TEMPLATE")
fi
if [ -n "$DAGLOCK_VAULT_TEMPLATE" ]; then
    DOCKER_ARGS+=(--daglock-vault-template "$DAGLOCK_VAULT_TEMPLATE")
fi
if [ -n "$TREASURY_PUBKEY" ]; then
    DOCKER_ARGS+=(--treasury-pubkey "$TREASURY_PUBKEY")
fi

docker run "${DOCKER_ARGS[@]}"

echo "Waiting for startup..."
sleep 5

if curl -sf http://localhost:$PORT/v1/health >/dev/null 2>&1; then
    echo "Indexer running on port $PORT (network: $NETWORK)"
    echo "API: http://localhost:$PORT/v1"
else
    echo "ERROR: Indexer failed to start. Check logs:"
    docker logs daglock-indexer --tail 20
    exit 1
fi
