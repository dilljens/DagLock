const { chromium } = require("playwright");
const fs = require("fs");

(async () => {
	const browser = await chromium.launch();
	const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });

	fs.mkdirSync("/home/dillon/_code/DagLock/screenshots", { recursive: true });

	// Screenshot 1: Swap page
	await page.goto("http://localhost:5173/swap");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/swap-not-connected.png", fullPage: true });
	console.log("1/6 swap page");

	// Screenshot 2: Dashboard
	await page.goto("http://localhost:5173/");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/dashboard.png", fullPage: true });
	console.log("2/6 dashboard");

	// Screenshot 3: Offers
	await page.goto("http://localhost:5173/offers");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/offers.png", fullPage: true });
	console.log("3/6 offers");

	// Screenshot 4: Security
	await page.goto("http://localhost:5173/security");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/security.png", fullPage: true });
	console.log("4/6 security");

	// Screenshot 5: Stats
	await page.goto("http://localhost:5173/stats");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/stats.png", fullPage: true });
	console.log("5/6 stats");

	// Screenshot 6: Blog
	await page.goto("http://localhost:5173/blog");
	await page.waitForLoadState("networkidle");
	await page.screenshot({ path: "/home/dillon/_code/DagLock/screenshots/blog.png", fullPage: true });
	console.log("6/6 blog");

	await browser.close();
	console.log("All screenshots done");
})();
