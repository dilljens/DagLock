#!/bin/bash
# DagLock Testnet Deployment Script
# Usage: ./scripts/deploy-testnet.sh

set -e

echo "=== DagLock Testnet Deployment ==="

# Check prerequisites
echo "Checking prerequisites..."
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install Rust."; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Error: node not found. Install Node.js."; exit 1; }

# Build indexer
echo "Building indexer..."
cargo build --release -p daglock-indexer

# Extract template hashes
echo "Extracting template hashes..."
cargo test -p daglock-contracts -- --nocapture template_hash_is_deterministic 2>&1 | grep -E "template_hash|hash" || echo "Check test output for hashes"

# Install bot dependencies
echo "Installing bot dependencies..."
cd bot && npm install && cd ..

# Install web dependencies
echo "Installing web dependencies..."
cd web && npm install && npm run build && cd ..

echo ""
echo "=== Build Complete ==="
echo ""
echo "Next steps:"
echo "1. Copy target/release/daglock-indexer to your server"
echo "2. Set up environment variables (see docs/DEPLOYMENT.md)"
echo "3. Run the indexer: ./daglock-indexer"
echo "4. Set up nginx reverse proxy"
echo "5. Start the bot: cd bot && BOT_TOKEN=xxx node src/index.js"
echo ""
echo "For detailed instructions, see docs/DEPLOYMENT.md"
