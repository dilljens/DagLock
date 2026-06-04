# Deployment: Railway + Cloudflare Pages

## What is an indexer?

The indexer is the DagLock backend. It does:

| Function | What it does |
|----------|-------------|
| **REST API** | 30+ endpoints for escrows, offers, reputation, messaging, jury, vouching |
| **Database** | Stores all escrow records, reputation data, messages, votes |
| **Auth** | Verifies Schnorr signatures before allowing state-changing operations |
| **Reconciliation** | Background loop that expires escrows/offers and detects on-chain UTXOs |
| **wRPC listener** | Connects to a Kaspa node to detect lock transactions (stub — runs in offline mode without it) |

Without an indexer, users would have to scan the Kaspa blockchain manually to find their escrows. The indexer makes everything searchable.

## Why you need deployment options

| Option | Best for | Cost | Setup time | Maintenance |
|--------|----------|------|------------|-------------|
| **Railway** | Solo dev, quick launch | ~$5-10/mo | 10 min | Near-zero |
| **VPS + Docker** | Full control, production | ~$10-20/mo VPS | 1 hour | Moderate |
| **Render** | Team deploys from GitHub | ~$7-15/mo | 15 min | Near-zero |
| **Self-hosted** | Privacy/regulation | Hardware cost | Days | High |
| **No indexer (P2P only)** | Future — not yet built | Free | Not available | N/A |

Railway is the recommendation because it auto-detects the Dockerfile, handles SSL, and provides a public URL instantly.

---

## Architecture

```
Cloudflare DNS
  daglock.com ──▶ Cloudflare Pages (static web UI)
  api.daglock.io ──▶ Railway (indexer backend)
                          │
                     SQLite volume (persistent)
                          │
                     Kaspa node (optional wRPC)
```

## Prerequisites

- GitHub account
- Railway account (github.com/railwayapp)
- Cloudflare account (cloudflare.com)
- Domain name (daglock.com — register via Cloudflare)

## Step 1: DNS (Cloudflare)

Add your domain to Cloudflare. Create these DNS records:

| Type | Name | Value |
|------|------|-------|
| A | `@` (daglock.com) | `192.0.2.1` (placeholder — Cloudflare Pages provides the real IP after deploy) |
| CNAME | `api` | Your Railway URL (looks like `daglock-indexer.up.railway.app`) |

After deploying (Step 3), update the API URL in Cloudflare Pages.

## Step 2: Web UI (Cloudflare Pages)

```bash
# 1. Build the web UI
cd web
npm install
npm run build        # produces web/dist/

# 2. In vite.config.ts, remove the proxy for production:
#    The proxy is only needed for local dev. In production,
#    the web UI calls api.daglock.io directly.
```

```ts
// web/vite.config.ts — production build
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
  },
});
```

Push the `web/` directory to a GitHub repo, then in Cloudflare Pages dashboard:

1. New Project → Connect GitHub repo
2. Build settings:
   - Build command: `npm run build`
   - Output directory: `web/dist`
   - Root directory: `web/`
3. Deploy

## Step 3: Indexer (Railway)

Push the entire repo to GitHub. In Railway dashboard:

1. New Project → Deploy from GitHub repo
2. Railway detects the Dockerfile automatically
3. Add environment variables:

| Variable | Value | Purpose |
|----------|-------|---------|
| `DAGLOCK_MESSAGE_KEY` | `openssl rand -hex 32` | AES-256-GCM encryption key |
| `PORT` | `8443` | Railway maps 8443 to its public port |
| `RUST_LOG` | `info` | Logging level |

4. Add a volume for persistent SQLite data:
   - Mount point: `/data`
   - Name: `daglock-data`
   - Update the start command: `--database-url sqlite:/data/daglock.db`

5. Deploy. Railway builds the Docker image and starts the indexer.

## Step 4: Wire them together

In Cloudflare Pages → your project → Custom domains:
- Add `daglock.com`
- Add `api.daglock.io` → In the Cloudflare dashboard, create a CNAME from `api.daglock.io` to your Railway URL

In the web UI, update `api.ts` to point to `https://api.daglock.io` instead of `localhost`:

```ts
// web/src/api.ts — the fetch URL is set by environment or build variable
// For Cloudflare Pages, use an environment variable:
// API_URL=https://api.daglock.io
// The web UI reads it at build time
```

## Cost breakdown

| Service | Component | Monthly cost |
|---------|-----------|-------------|
| Cloudflare (free tier) | DNS + Pages | $0 |
| Railway (starter) | Indexer + SQLite | $5-10 |
| Domain registration | daglock.com | ~$9/year |
| **Total** | | **~$5-10/mo** |

## Alternative: All-in-one VPS (control + lower cost)

If you prefer a VPS:

```bash
# Hetzner CX22 ($8/mo) or similar
ssh root@your-server

# Install Docker
curl -fsSL https://get.docker.com | sh

# Clone and deploy
git clone https://github.com/your-org/daglock
cd daglock
DAGLOCK_MESSAGE_KEY=$(openssl rand -hex 32) ./scripts/deploy-mainnet.sh

# Set up nginx (the config at nginx.conf serves both web UI and API)
# Replace api.daglock.io with daglock.com
cp nginx.conf /etc/nginx/sites-available/daglock
ln -s /etc/nginx/sites-available/daglock /etc/nginx/sites-enabled/
certbot --nginx -d daglock.com -d api.daglock.io
```

## Testnet launch plan

For users to test without running anything:

```
1. Deploy indexer to Railway (offline mode, no wRPC)
2. Deploy web UI to Cloudflare Pages
3. Users visit daglock.com and interact with the API
4. Users can:
   - Create escrows manually (wRPC creates manual active-trigger)
   - Browse offers
   - Check reputation
   - Use the CLI: cargo install daglock-cli
```

No Kaspa node required for basic functionality. The only thing that doesn't work offline is automatic UTXO detection — users can still test the UI, reputation, offers, messaging, and jury system.

## Railway config file

Create `railway.json` in the project root:

```json
{
  "build": {
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  },
  "deploy": {
    "startCommand": "daglock-indexer --host 0.0.0.0 --port 8443",
    "healthcheckPath": "/v1/health",
    "restartPolicyType": "ON_FAILURE"
  }
}
```
