#!/usr/bin/env bash
set -euo pipefail

# DagLock local Kaspa simnet launcher
# Prerequisites: rusty-kaspa built and available in PATH or a known location

KASPAD="${KASPAD_BIN:-kaspad}"
DATA_DIR="${DAGLOCK_DATA_DIR:-/tmp/daglock-simnet}"
RPC_PORT="${RPC_PORT:-18110}"
MINING_ADDR="${MINING_ADDR:-}"
NETWORK="simnet"

echo "==> DagLock Local Testnet"
echo "    Network:  ${NETWORK}"
echo "    Data dir: ${DATA_DIR}"
echo "    RPC port: ${RPC_PORT}"

# Ensure data dir exists
mkdir -p "${DATA_DIR}"

# Start kaspad in simnet mode with UTXO index
exec "${KASPAD}" \
    --${NETWORK} \
    --appdir "${DATA_DIR}" \
    --rpclisten "127.0.0.1:${RPC_PORT}" \
    --utxoindex \
    ${MINING_ADDR:+--miningaddr "${MINING_ADDR}"}
