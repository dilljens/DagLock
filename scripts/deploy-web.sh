#!/bin/bash
# Deploy DagLock web UI to Cloudflare Pages
# Usage: ./scripts/deploy-web.sh
set -euo pipefail

source .env.cloudflare

# Get account ID
ACCOUNT_ID=$(curl -sf "https://api.cloudflare.com/client/v4/zones?name=daglock.com" \
	-H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" |
	python3 -c "import sys,json; print(json.load(sys.stdin)['result'][0]['account']['id'])")

# Trigger deployment
DEPLOY_ID=$(curl -sf -X POST \
	"https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/pages/projects/daglock/deployments" \
	-H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
	-H "Content-Type: application/json" \
	-d '{"branch":"main"}' |
	python3 -c "import sys,json; print(json.load(sys.stdin)['result']['id'])")

echo "Deployment triggered: $DEPLOY_ID"

# Wait for completion
echo "Waiting for deployment..."
for i in {1..60}; do
	STATUS=$(curl -sf "https://api.cloudflare.com/client/v4/accounts/$ACCOUNT_ID/pages/projects/daglock/deployments/$DEPLOY_ID" \
		-H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" |
		python3 -c "import sys,json; print(json.load(sys.stdin)['result']['latest_stage']['status'])" 2>/dev/null)

	if [ "$STATUS" = "success" ]; then
		echo "✅ Deployment successful!"
		echo "https://daglock.com"
		exit 0
	elif [ "$STATUS" = "failure" ]; then
		echo "❌ Deployment failed"
		exit 1
	fi

	sleep 5
done

echo "⏱️ Deployment timed out"
