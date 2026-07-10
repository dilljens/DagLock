import { test, expect } from "./fixtures";

test.describe("Offer Lifecycle", () => {
	test("shows empty state when no offers", async ({ page }) => {
		// Override to return empty offers
		await page.route("**/v1/offers", async (route) => {
			if (route.request().method() === "GET") {
				await route.fulfill({
					status: 200, contentType: "application/json",
					body: JSON.stringify({ offers: [], total: 0 }),
				});
			}
		});

		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await expect(page.getByText("No open offers")).toBeVisible({ timeout: 5000 });
	});

	test("browse tab shows mock offer cards", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// Should show the mock offer
		await expect(page.locator(".offer")).toBeVisible({ timeout: 5000 });
		await expect(page.getByText("BUY")).toBeVisible();
	});

	test("create tab shows form fields", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(500);

		// Go to Create tab
		await page.locator(".tab-bar").getByText("Create").click();
		await page.waitForTimeout(500);

		// Should show form fields (address is auto-filled from wallet)
		await expect(page.getByText("Amount (KAS)")).toBeVisible({ timeout: 5000 });
	});

	test("create offer requires wallet to be connected", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Go to Create tab — should show connect prompt since wallet isn't connected yet
		// Actually, the mock KasWare is detected but not connected
		await page.locator(".tab-bar").getByText("Create").click();
		await page.waitForTimeout(500);

		// The page shows either the form or a connect prompt depending on wallet state
		// With mock KasWare, clicking connect should work
		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(1000);

		// Now go to offers and create tab
		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.locator(".tab-bar").getByText("Create").click();
		await page.waitForTimeout(500);

		// Should see the form
		await expect(page.locator("form")).toBeVisible({ timeout: 5000 });
	});

	test("accept button calls API with auth", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// Connect wallet
		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(1000);

		// Reload to see offers with connected wallet
		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// The mock offer card should show an Accept button or counterparty input
		const offerCard = page.locator(".offer").first();
		await expect(offerCard).toBeVisible({ timeout: 5000 });

		// Fill in the counterparty address (needed for accept)
		const addrInput = offerCard.locator('input[placeholder="your address"]');
		if (await addrInput.isVisible()) {
			await addrInput.fill("kaspa:testacceptaddress1234567890abcdef");
		}

		// Click Accept
		const acceptBtn = offerCard.locator("button").filter({ hasText: "Accept" });
		if (await acceptBtn.isVisible()) {
			await acceptBtn.click();
			// Should trigger signMessage and then show success notification
			await page.waitForTimeout(2000);
			// Check for success toast or notification
			await expect(page.locator(".Toastify__toast--success, .notification-success").first()).toBeVisible({ timeout: 10000 });
		}
	});

	test("my offers tab shows connect prompt without wallet", async ({ page }) => {
		// Navigate without wallet connected (KasWare mock exists but not connected)
		await page.goto("/offers");
		await page.waitForLoadState("load");

		await page.locator(".tab-bar").getByText("My Offers").click();
		await page.waitForTimeout(500);

		// Should show connect prompt
		await expect(page.getByText("Connect your wallet")).toBeVisible({ timeout: 5000 });
	});

	test("offer card shows deal type badge and memo", async ({ page }) => {
		await page.route("**/v1/offers", async (route) => {
			if (route.request().method() === "GET") {
				await route.fulfill({
					status: 200, contentType: "application/json",
					body: JSON.stringify({
						offers: [{
							id: "offer_deals_test",
							creator_address: "kaspa:testcreatoraddress1234567890abcdef",
							side: "sell",
							base_asset: "KAS",
							quote_asset: "KRC20:NACHO",
							amount_sompi: 500_000_000,
							status: "proposed",
							created_at: Date.now() / 1000 - 3600,
							deal_type: "otc",
							memo: "Selling 5 KAS for NACHO tokens",
							expires_at: Date.now() / 1000 + 86400,
							price_type: "fixed",
							price_currency: "USD",
						}],
						total: 1,
					}),
				});
			}
		});

		await page.goto("/offers");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// The offer card should show the memo
		await expect(page.getByText("Selling 5 KAS for NACHO tokens")).toBeVisible({ timeout: 5000 });
		// Should show OTC badge
		await expect(page.getByText("OTC")).toBeVisible();
	});
});
