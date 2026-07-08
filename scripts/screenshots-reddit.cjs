// DagLock Reddit Tutorial Screenshots
// Usage: cd web && node ../scripts/screenshots-reddit.cjs
// Requires: Playwright browsers, Vite dev server on :5173, indexer on :8443

const { chromium } = require("@playwright/test");
const fs = require("fs");

const SITE = "http://127.0.0.1:5173";
const OUT = "/home/dillon/_code/DagLock/screenshots";

const BUYER_ADDR = "kaspa:qtqwyqtmgczzjmj44vjzy";
const SELLER_ADDR = "kaspa:qjdpca9zm8aafdue2q0zn";

(async () => {
	const browser = await chromium.launch({ headless: true });
	const context = await browser.newContext({
		viewport: { width: 1280, height: 800 },
	});

	// Dismiss onboarding
	await context.addInitScript(() => {
		localStorage.setItem("daglock_onboarded", "true");
	});

	const page = await context.newPage();
	fs.mkdirSync(OUT, { recursive: true });

	// ─── Screenshot 1: Dashboard with testnet banner ───
	console.log("[1/4] Capturing dashboard with testnet banner...");
	await page.goto(SITE + "/", { waitUntil: "networkidle", timeout: 15000 });
	await page.waitForTimeout(2000);
	await page.screenshot({
		path: OUT + "/reddit-01-dashboard.png",
		fullPage: false,
	});
	console.log("  ✓ reddit-01-dashboard.png");

	// ─── Screenshot 2: Manual mode connected with buyer address ───
	console.log("[2/4] Capturing manual mode with address...");
	await page.goto(SITE + "/", { waitUntil: "networkidle", timeout: 15000 });
	await page.waitForTimeout(1000);

	// Click the manual mode toggle — look for "Use manual mode" or similar text
	try {
		// Try clicking the manual mode button in the sidebar footer
		const manualBtn = page.locator('button:has-text("Use manual mode")');
		if (await manualBtn.isVisible({ timeout: 3000 }).catch(() => false)) {
			await manualBtn.click();
			await page.waitForTimeout(500);
		}

		// Fill in the address input
		const addrInput = page.locator('input[placeholder*="kaspa:"]');
		if (await addrInput.isVisible({ timeout: 2000 }).catch(() => false)) {
			await addrInput.fill(BUYER_ADDR);
			await page.waitForTimeout(300);
			// Click "Set Address" button
			const setBtn = page.locator('button:has-text("Set Address")');
			if (await setBtn.isVisible({ timeout: 2000 }).catch(() => false)) {
				await setBtn.click();
				await page.waitForTimeout(1000);
			}
		}
	} catch (e) {
		console.log("  ⚠ Manual mode click via text failed, trying via evaluate...");
		// Fallback: try to set manual address via localStorage + reload
		await page.evaluate((addr) => {
			localStorage.setItem("daglock_manual_address", addr);
		}, BUYER_ADDR);
		await page.reload({ waitUntil: "networkidle" });
		await page.waitForTimeout(1000);
	}

	await page.screenshot({
		path: OUT + "/reddit-02-manual-mode.png",
		fullPage: false,
	});
	console.log("  ✓ reddit-02-manual-mode.png");

	// ─── Screenshot 3: Create escrow form with sample data ───
	console.log("[3/4] Capturing create escrow form...");
	await page.goto(SITE + "/escrows", { waitUntil: "networkidle", timeout: 15000 });
	await page.waitForTimeout(1500);

	// Click the Create tab
	try {
		const createTab = page.locator('button.tab-btn:has-text("Create")');
		if (await createTab.isVisible({ timeout: 3000 }).catch(() => false)) {
			await createTab.click();
			await page.waitForTimeout(1000);
		}
	} catch (e) {
		console.log("  ⚠ Could not click Create tab:", e.message);
	}

	// Fill in the form
	try {
		// Amount field — look for an input[type=number] in the create form
		const amountInput = page.locator('input[type="number"]').first();
		if (await amountInput.isVisible({ timeout: 2000 }).catch(() => false)) {
			await amountInput.fill("100");
		}

		// Seller address field
		const sellerInput = page.locator('input[placeholder*="kaspa:"]').first();
		if (await sellerInput.isVisible({ timeout: 2000 }).catch(() => false)) {
			await sellerInput.fill(SELLER_ADDR);
		}

		await page.waitForTimeout(500);
	} catch (e) {
		console.log("  ⚠ Could not fill form:", e.message);
	}

	await page.screenshot({
		path: OUT + "/reddit-03-create-escrow.png",
		fullPage: false,
	});
	console.log("  ✓ reddit-03-create-escrow.png");

	// ─── Screenshot 4: Escrow list showing settled + active ───
	console.log("[4/4] Capturing escrows list...");
	await page.goto(SITE + "/escrows", { waitUntil: "networkidle", timeout: 15000 });
	await page.waitForTimeout(2000);

	// Try to click My Escrows tab
	try {
		const myEscrowsTab = page.locator('button.tab-btn:has-text("My Escrows")');
		if (await myEscrowsTab.isVisible({ timeout: 2000 }).catch(() => false)) {
			await myEscrowsTab.click();
			await page.waitForTimeout(2000);
		}
	} catch (e) {
		console.log("  ⚠ Could not click My Escrows tab:", e.message);
	}

	await page.screenshot({
		path: OUT + "/reddit-04-settled.png",
		fullPage: false,
	});
	console.log("  ✓ reddit-04-settled.png");

	await browser.close();
	console.log("\nDone — 4 screenshots captured.");
})().catch((e) => {
	console.error("Fatal error:", e);
	process.exit(1);
});
