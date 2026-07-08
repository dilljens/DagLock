#!/bin/bash
# Deploy DagLock web UI to Cloudflare Pages
# 
# Two methods:
#   1. GitHub Actions (recommended) — push to main triggers auto-deploy via
#      .github/workflows/deploy-web.yml. Requires CLOUDFLARE_API_TOKEN set
#      as a GitHub repository secret.
#
#   2. Manual (this script) — uses npx wrangler to deploy from local machine.
#      Requires .env.cloudflare with CLOUDFLARE_API_TOKEN.
#
# Usage:
#   ./scripts/deploy-web.sh          # deploy from local (uses npx wrangler)
#   git push origin main             # deploy via GitHub Actions
#
set -euo pipefail

source .env.cloudflare

cd web

echo "=== Building web UI ==="
npm run build

echo ""
echo "=== Deploying to Cloudflare Pages ==="
npx --yes wrangler@latest pages deploy dist/ --project-name=daglock --branch main

echo ""
echo "=== Warming CDN cache (pre-fetching new chunks) ==="
# Get the new chunk names from the built output
for f in dist/assets/*.js; do
  name=$(basename "$f")
  echo "  Warming: /assets/$name"
  curl -s -o /dev/null "https://daglock.com/assets/$name" &
done
wait
echo "  ✅ Cache warmed"

echo ""
echo "✅ Deploy triggered. Check GitHub Actions for status:"
echo "   https://github.com/dilljens/DagLock/actions"
echo "   https://daglock.com"
