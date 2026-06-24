#!/usr/bin/env bash
# DagLock Laptop kaspad Setup (Testnet-12)
# Run this on your laptop:
#   bash scripts/setup-laptop-kaspad.sh
set -euo pipefail

echo "=== DagLock Laptop kaspad Setup ==="
echo ""

# ── Step 1: Clone rusty-kaspa master ──
if [ -d /tmp/rusty-kaspa ]; then
  echo "Removing previous clone..."
  rm -rf /tmp/rusty-kaspa
fi

echo "[1/4] Cloning rusty-kaspa (master)..."
git clone --depth 1 https://github.com/kaspanet/rusty-kaspa /tmp/rusty-kaspa

# ── Step 2: Apply testnet-12 patch ──
echo "[2/4] Applying testnet-12 patch..."
cd /tmp/rusty-kaspa
sed -i 's/Some(10) => TESTNET_PARAMS,/Some(10) | Some(12) => TESTNET_PARAMS,/' consensus/core/src/config/params.rs
echo "  Patched: testnet-12 is now supported"

# ── Step 3: Build kaspad ──
echo "[3/4] Building kaspad (release)..."
echo "  This takes ~20-25 min. Go grab a coffee."
cargo build --release --bin kaspad
echo "  Build complete!"

# ── Step 4: Print instructions ──
echo ""
echo "[4/4] Setup complete!"
echo ""
echo "============================================"
echo "  TO START KASPAD:"
echo "============================================"
echo ""
echo "  /tmp/rusty-kaspa/target/release/kaspad \\"
echo "    --testnet --netsuffix=12 --utxoindex"
echo ""
echo "  IBD will take ~30-60 min. wRPC opens on port 17210."
echo ""
echo "============================================"
echo "  TO CONNECT VPS TO YOUR LAPTOP (Tailscale):"
echo "============================================"
echo ""
echo "  1. On laptop:  tailscale up"
echo "  2. On VPS:     /opt/daglock-indexer/toggle-wrpc.sh on \\"
echo "                   ws://\$(tailscale ip -4):17210"
echo "  3. On VPS:     curl http://127.0.0.1:8443/v1/health"
echo ""
echo "============================================"
echo "  TO TEST LOCALLY (no VPS needed):"
echo "============================================"
echo ""
echo "  Run the indexer locally against your kaspad:"
echo "  cargo run -p daglock-indexer -- \\"
echo "    --network testnet-12 \\"
echo "    --wrpc-url ws://127.0.0.1:17210 \\"
echo "    --daglock-kas-template 30876e3ea42d0e23bb0980f3fd97ae8807e9c70f \\"
echo "    --daglock-krc20-template ae0946e4a9bd4a7585e6bf9135de38083cb11c85 \\"
echo "    --daglock-reputation-template 65c54102c64a331414b602760cbd76efac3d69df"
echo ""
