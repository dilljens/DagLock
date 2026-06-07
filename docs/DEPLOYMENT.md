# DagLock Deployment Guide

## Latest: Docker (Production)

```bash
# 1. Generate a message encryption key
DAGLOCK_MESSAGE_KEY=$(openssl rand -hex 32)
export DAGLOCK_MESSAGE_KEY

# 2. Deploy with Docker
./scripts/deploy-mainnet.sh

# Or custom settings:
CORS_ORIGIN=https://daglock.io DAGLOCK_MESSAGE_KEY=$KEY ./scripts/deploy-mainnet.sh
```

## Configuration Reference

| Env / Flag | Required | Default | Description |
|------------|----------|---------|-------------|
| DAGLOCK_MESSAGE_KEY | YES | (ephemeral dev key) | 64 hex chars for AES-256-GCM |
| --wrpc-url | Yes* | — | Kaspa node wRPC URL (ws://host:port) |
| --network | Yes | testnet-12 | mainnet/testnet-N/simnet/devnet |
| --allow-mainnet | No | false | Must set for mainnet (safety) |
| --cors-origin | No | * | CORS origin (set to domain in prod) |
| --db-type | No | sqlite | sqlite or postgres |
| --log-level | No | info | debug/info/warn/error |

*Required for live UTXO detection. Without it, runs in offline reconciliation mode.

## Quick Start (Dev)

```bash
# Prerequisites
# - Rust toolchain (stable)
# - Node.js 18+ (for bot/web)
# - 4GB+ RAM recommended

# 1. Build the Indexer

```bash
# Clone the repo
git clone https://github.com/your-org/daglock.git
cd daglock

# Build release binary
cargo build --release -p daglock-indexer

# Binary will be at: target/release/daglock-indexer
```

## 2. Configure Environment

```bash
# Create config directory
mkdir -p ~/.config/daglock

# Create environment file
cat > ~/.config/daglock/env << 'EOF'
# Database
DATABASE_URL=sqlite:$HOME/.local/share/daglock/daglock.db

# Network
NETWORK=testnet-12

# wRPC (use public testnet resolver or your own node)
WRPC_URL=wss://tn12.kaspa.org:16210

# Template hashes (from contracts compilation)
DAGLOCK_KAS_TEMPLATE=<hash-from-compile>
DAGLOCK_KRC20_TEMPLATE=<hash-from-compile>

# Server
HOST=0.0.0.0
PORT=8443
LOG_LEVEL=info
EOF
```

## 3. Initialize Database

```bash
# Create data directory
mkdir -p ~/.local/share/daglock

# The indexer will auto-create tables on first run
```

## 4. Run the Indexer

```bash
# Load environment
source ~/.config/daglock/env

# Run the indexer
./target/release/daglock-indexer

# Or with systemd (see below)
```

## 5. Systemd Service (Production)

```bash
cat > /etc/systemd/system/daglock.service << 'EOF'
[Unit]
Description=DagLock Indexer
After=network.target

[Service]
Type=simple
User=daglock
WorkingDirectory=/home/daglock
ExecStart=/home/daglock/daglock-indexer
EnvironmentFile=/home/daglock/.config/daglock/env
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Create service user
sudo useradd -r -s /bin/false daglock
sudo mkdir -p /home/daglock/.config/daglock
sudo cp ~/.config/daglock/env /home/daglock/.config/daglock/
sudo chown -R daglock:daglock /home/daglock

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable daglock
sudo systemctl start daglock
```

## 6. Nginx Reverse Proxy (HTTPS)

```nginx
server {
    listen 443 ssl http2;
    server_name api.daglock.io;

    ssl_certificate /etc/letsencrypt/live/api.daglock.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.daglock.io/privkey.pem;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s;

    location / {
        limit_req zone=api burst=20 nodelay;
        
        proxy_pass http://127.0.0.1:8543;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # WebSocket support (for future real-time updates)
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## 7. Deploy Telegram Bot

```bash
cd bot
npm install

# Run with systemd
BOT_TOKEN=your_token INDEXER_URL=http://localhost:8543 node src/index.js
```

## 8. Deploy Web UI

```bash
cd web
npm install
npm run build

# Copy dist/ to nginx or serve statically
cp -r dist/* /var/www/daglock.io/
```

## 9. Verify Deployment

```bash
# Health check
curl https://api.daglock.io/v1/health

# Stats
curl https://api.daglock.io/v1/stats

# Test CLI
./target/release/daglock-cli --api-url https://api.daglock.io status --id esc_test
```

## 10. Monitor Logs

```bash
# Indexer logs
journalctl -u daglock -f

# Or if running manually
RUST_LOG=info ./target/release/daglock-indexer
```

---

## Template Hash Extraction

After compiling the covenants, extract template hashes:

```bash
# The hashes will be printed during compilation
# Or extract from the compiled contract:
cargo test -p daglock-contracts -- --nocapture template_hash_is_deterministic
```

---

## Troubleshooting

### Connection refused to wRPC
- Check if testnet node is accessible: `wscat -c wss://tn12.kaspa.org:16210`
- Try alternative endpoints or run your own node

### Database locked
- Only one indexer instance can run per database file
- Check for competing processes: `ps aux | grep daglock`

### High memory usage
- SQLite can use significant memory with large datasets
- Consider PostgreSQL for production (>1000 escrows)
