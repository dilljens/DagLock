# Deploy DagLock — Railway + Cloudflare Pages

> Estimated time: 30 minutes · Cost: ~$5-10/mo

## Architecture

```
                     
                         Cloudflare DNS            
                       daglock.com           
                       api.daglock.io           
                     
                                       
              
                                                         
               
      Cloudflare Pages          Railway                 
        (web UI)              (indexer backend)         
        daglock.com          api.daglock.io            
                                                        
       React SPA             Rust Axum API              
       api.ts →          SQLite volume              
       api.daglock.io        DAGLOCK_MESSAGE_KEY        
               
                                                           
         User's browser                                     
                                                           
              
```

---

## Step 1: Prerequisites

| Account | Sign up at | Cost |
|---------|-----------|------|
| GitHub | github.com | Free |
| Railway | railway.com | Free (no card for first deploy) |
| Cloudflare | cloudflare.com | Free |

**Domain:** You need `daglock.com` (or any domain). Register through Cloudflare (~$9/yr) or wherever you prefer.

---

## Step 2: GitHub

Push the repo:

```bash
# In the daglock directory
git remote add origin https://github.com/YOUR_USERNAME/daglock.git
git push -u origin main
```

---

## Step 3: Deploy the Indexer (Railway) — 5 minutes

### 3a. Create the service

1. Go to [railway.com/dashboard](https://railway.com/dashboard)
2. Click **New Project** → **Deploy from GitHub repo**
3. Select your `daglock` repo
4. Railway detects the `Dockerfile` automatically
5. Click **Deploy**

### 3b. Add environment variables

After the first deploy (it will fail on the first try — that's expected), go to **Variables** and add:

| Variable | Value | How to generate |
|----------|-------|-----------------|
| `DAGLOCK_MESSAGE_KEY` | (64 hex chars) | Run `openssl rand -hex 32` in your terminal |
| `PORT` | `8443` | Railway maps this port automatically |
| `RUST_LOG` | `info` | Logging level |

### 3c. Add a volume (keeps your database between deploys)

1. Go to the **Volumes** tab
2. Click **Add Volume**
3. Name: `daglock-data`
4. Mount path: `/data`
5. Size: 1GB (plenty for SQLite)

### 3d. Update the start command

In the **Deploy** tab → **Start Command**, change to:

```
daglock-indexer --host 0.0.0.0 --port 8443 --database-url sqlite:/data/daglock.db
```

### 3e. Verify it's running

Railway gives you a URL like `daglock-indexer.up.railway.app`. Visit:

```
https://daglock-indexer.up.railway.app/v1/health
```

You should see: `{"status":"ok","version":"0.1.0","uptime_seconds":...}`

---

## Step 4: Deploy the Web UI (Cloudflare Pages) — 5 minutes

### 4a. Configure the API URL

Create `web/.env` with the Railway URL:

```bash
# web/.env
VITE_API_URL=https://daglock-indexer.up.railway.app
```

Update `web/src/api.ts` to read from the env var:

```ts
const API_BASE = import.meta.env.VITE_API_URL || "";
// All fetch calls become: fetch(`${API_BASE}/v1/health`)
```

### 4b. Deploy to Cloudflare Pages

1. Go to [dash.cloudflare.com](https://dash.cloudflare.com) → **Pages**
2. Click **Create a project** → **Connect to Git**
3. Select your `daglock` repo
4. Build settings:
   - **Build command:** `npm run build`
   - **Build output:** `dist`
   - **Root directory:** `web`
5. **Environment variables (advanced):**
   - `VITE_API_URL`: `https://daglock-indexer.up.railway.app`
6. Click **Save and Deploy**

### 4c. Add your custom domain

1. In Cloudflare Pages → your project → **Custom domains**
2. Add `daglock.com`
3. Cloudflare automatically adds the DNS records

---

## Step 5: DNS (Cloudflare) — 2 minutes

Add your domain to Cloudflare if you haven't already:

1. Go to Cloudflare Dashboard → **Add a Site**
2. Enter your domain name
3. Cloudflare scans your existing DNS records
4. Change your domain's nameservers to the ones Cloudflare provides (at your registrar)
5. Wait 5-10 minutes for DNS to propagate

Once done, `daglock.com` serves the web UI and the web UI calls `https://daglock-indexer.up.railway.app` for the API.

---

## Step 6: Verify everything

```bash
# The web UI
curl https://daglock.com

# The API (proxied from the web UI)
curl https://daglock.com/v1/health

# Or direct to Railway
curl https://daglock-indexer.up.railway.app/v1/health
```

Open `https://daglock.com` in your browser. You should see the DagLock dashboard.

---

## Ongoing maintenance

### Railway
- **Logs:** Railway Dashboard → your project → **Deployments** → **View logs**
- **Restart:** Click **Redeploy** on the latest deployment
- **Scaling:** Railway auto-scales. Upgrade your plan if you need more RAM.

### Cloudflare Pages
- **Auto-deploy:** Every push to `main` rebuilds and deploys
- **Preview deploys:** Pull requests get preview URLs automatically

### Updating
```bash
git add -A
git commit -m "fix: something"
git push origin main
# Railway auto-rebuilds the indexer
# Cloudflare Pages auto-rebuilds the web UI
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Indexer won't start | Missing `DAGLOCK_MESSAGE_KEY` | Add it in Railway Variables |
| 502 on health check | Port mismatch | Ensure `PORT=8443` in Railway Variables |
| Web UI shows blank page | Wrong API URL | Check `VITE_API_URL` in Cloudflare Pages env vars |
| "Connection refused" | Indexer still building | Wait 2-3 minutes for Docker build |
| Offers don't appear | Indexer running but no data | Create some via the web UI |
| 404 on API routes | Old version of indexer | Railway auto-deploys — check Deployments tab |

---

## How people use it

Once deployed, users can test everything without running anything:

1. Visit `https://daglock.com`
2. Create offers and escrows
3. Check reputation scores
4. Send messages on escrows
5. Register as a juror
6. Link Telegram handles

The only thing that doesn't work without a Kaspa node is automatic on-chain UTXO detection. But the entire UX — offers, reputation, messaging, jury, vouching — works fully against the indexer. This is enough for testnet feedback.
