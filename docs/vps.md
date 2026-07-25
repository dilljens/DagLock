# VPS — DagLock Deployment

> DagLock runs on a shared OVHcloud VPS alongside other projects.
> See `/home/dillon/MEGA/FerrumEng/VPS.md` for full infrastructure details (ports, systemd services, SSL, etc.)

## Quick Reference

| Field | Value |
|-------|-------|
| **Provider** | OVHcloud US |
| **IP** | `40.160.241.74` |
| **SSH user** | `ubuntu` |
| **SSH key** | Your personal key (`id_ed25519`) |
| **Fallback password** | `raspi9000` |
| **OS** | Ubuntu 26.04 LTS |
| **Specs** | 4 vCore · 8 GB RAM · 75 GB NVMe · 400 Mbps |

## DagLock Services

| Service | What | Port |
|---------|------|------|
| `daglock-indexer.service` | Rust API (Rust indexer binary) | `:8443` → nginx `:443` |
| `daglock-bot.service` | Telegram bot (Node.js) | — (outbound only) |

## Locations

| Component | Path |
|-----------|------|
| **Indexer binary + DB** | `/opt/daglock-indexer/` |
| **Bot** | `/opt/daglock-bot/` |
| **SQLite database** | `/opt/daglock-indexer/daglock.db` |
| **Bot token** | `BOT_TOKEN` in `/etc/systemd/system/daglock-bot.service` |
| **SSL cert** | `/etc/letsencrypt/live/api.daglock.com/` (expires Sep 22, 2026) |

## nginx

```
api.daglock.com → nginx :443 → indexer :8443
```

Current config: needs `client_max_body_size 1m;` added for mainnet hardening.

## Common Commands

```bash
# SSH
ssh ubuntu@40.160.241.74

# View service status
systemctl status daglock-indexer
systemctl status daglock-bot

# Tail logs
journalctl -u daglock-indexer -f
journalctl -u daglock-bot -f

# Restart
systemctl restart daglock-indexer
systemctl restart daglock-bot

# Nginx
nginx -t && systemctl reload nginx

# Database location
ls -lh /opt/daglock-indexer/daglock.db

# Resource usage
htop
df -h /
free -h
```

## Updating the Bot Token

If you revoke and rotate the bot token in @BotFather:

```bash
# 1. Edit the systemd unit
sudo systemctl edit daglock-bot.service

# 2. Set:
# [Service]
# Environment=BOT_TOKEN=<new-token>

# 3. Restart
sudo systemctl daemon-reload
sudo systemctl restart daglock-bot
```

## Deploying Updates

```bash
# Build and deploy (adjust paths as needed)
cargo build --release -p daglock-indexer
sudo cp target/release/daglock-indexer /opt/daglock-indexer/
sudo systemctl restart daglock-indexer

# Bot
rsync -avz --exclude node_modules bot/ ubuntu@40.160.241.74:/opt/daglock-bot/
```

---

*See `/home/dillon/MEGA/FerrumEng/VPS.md` for all projects on this server and full service map.*
