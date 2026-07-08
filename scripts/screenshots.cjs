// DagLock Screenshot Tool
// Usage: cd web && node ../scripts/screenshots.cjs
// Requires: Playwright browsers installed (npx playwright install chromium)
// Requires: Vite dev server running on port 5173

const { chromium } = require("@playwright/test");
const fs = require("fs");

const SITE = "http://localhost:5173";
const OUT = "/home/dillon/_code/DagLock/screenshots";
const PAGES = [
	{ path: "/", name: "dashboard.png", wait: 3000 },
	{ path: "/offers", name: "offers.png", wait: 2000 },
	{ path: "/swap", name: "swap.png", wait: 2000 },
	{ path: "/stats", name: "stats.png", wait: 2000 },
	{ path: "/security", name: "security.png", wait: 2000 },
	{ path: "/blog", name: "blog.png", wait: 2000 },
	{ path: "/escrows", name: "escrows.png", wait: 2000 },
	{ path: "/vaults", name: "vaults.png", wait: 2000 },
	{ path: "/tokens", name: "tokens.png", wait: 2000 },
	{ path: "/merchant", name: "merchant.png", wait: 2000 },
	{ path: "/subscriptions", name: "subscriptions.png", wait: 2000 },
];

(async () => {
	const browser = await chromium.launch({ headless: true });
	const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });

	// Dismiss onboarding
	await context.addInitScript(() => localStorage.setItem("daglock_onboarded", "true"));

	const page = await context.newPage();
	fs.mkdirSync(OUT, { recursive: true });

	let i = 0;
	for (const { path, name, wait } of PAGES) {
		i++;
		try {
			await page.goto(SITE + path, { waitUntil: "networkidle", timeout: 15000 });
			await page.waitForTimeout(wait);
			await page.screenshot({ path: OUT + "/" + name, fullPage: true });
			console.log(`✓ [${i}/${PAGES.length}] ${name} (${path})`);
		} catch (e) {
			console.log(`✗ [${i}/${PAGES.length}] ${name} FAILED: ${e.message}`);
		}
	}

	await browser.close();
	console.log("Done");
})().catch((e) => {
	console.error(e);
	process.exit(1);
});
