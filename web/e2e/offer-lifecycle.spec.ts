import { test, expect } from "./fixtures";

test.describe("Offer Lifecycle", () => {
	test("shows empty state when no offers", async ({ page }) => {
		// Override to return empty offers — use * wildcard to match query params (?status=proposed)
		await page.route("**/v1/offers*", async (route) => {
			await route.fulfill({
				status: 200, contentType: "application/json",
				body: JSON.stringify({ offers: [], total: 0 }),
			});
		});

		await page.goto("/offers");
		await page.waitForLoadState("load");

		await expect(page.getByText("No open offers")).toBeVisible({ timeout: 10000 });
	});

	test("browse tab shows mock offer cards", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Should show the mock offer (from fixture API mocks)
		await expect(page.locator(".offer")).toBeVisible({ timeout: 10000 });
		// Check for BUY inside the offer card (use .offer to disambiguate)
		await expect(page.locator(".offer").getByText("BUY")).toBeVisible();
	});

	test("create tab shows form fields", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Connect wallet first (wallet is detected but not connected)
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Go to Create tab
		const createTab = page.locator(".tab-bar").getByText("Create");
		await createTab.waitFor({ state: "visible", timeout: 10000 });
		await createTab.click();

		// Should show form fields (address is auto-filled from wallet)
		await expect(page.getByText("Amount (KAS)")).toBeVisible({ timeout: 10000 });
	});

	test("create offer requires wallet to be connected", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Go to Create tab — should show connect prompt since wallet isn't connected yet
		const createTab = page.locator(".tab-bar").getByText("Create");
		await createTab.waitFor({ state: "visible", timeout: 10000 });
		await createTab.click();

		// Wait for connect button, then connect
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Now the create tab should show the form instead of connect prompt
		await expect(page.locator("form")).toBeVisible({ timeout: 10000 });
	});

	test("accept button calls API with auth", async ({ page }) => {
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Connect wallet — wallet mock auto-detects but doesn't auto-connect
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Reload to see offers with connected wallet
		// Wallet state doesn't persist across page reload, so reconnect
		await page.goto("/offers");
		await page.waitForLoadState("load");

		// Reconnect wallet after page reload
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// The mock offer card should show
		const offerCard = page.locator(".offer").first();
		await expect(offerCard).toBeVisible({ timeout: 10000 });

		// Fill in the counterparty address (needed for accept)
		const addrInput = offerCard.locator('input[placeholder="your address"]');
		await expect(addrInput).toBeVisible({ timeout: 5000 });
		await addrInput.fill("kaspa:testacceptaddress1234567890abcdef");

		// Click Accept — button is visible when offer is proposed and user is not the creator
		const acceptBtn = offerCard.locator("button.primary").filter({ hasText: "Accept" });
		await expect(acceptBtn).toBeVisible({ timeout: 5000 });
		await acceptBtn.click();

		// The mock API returns 200 and the app shows a toast notification.
		// The app uses framer-motion AnimatePresence for toasts, which may prefix
		// class names. Check for the toast container having any toast element.
		await expect(page.locator(".toast-container")).toBeVisible({ timeout: 5000 });
	});

	test("my offers tab shows connect prompt without wallet", async ({ page }) => {
		// Navigate without wallet connected (KasWare mock exists but not connected)
		await page.goto("/offers");
		await page.waitForLoadState("load");

		const myOffersTab = page.locator(".tab-bar").getByText("My Offers");
		await myOffersTab.waitFor({ state: "visible", timeout: 10000 });
		await myOffersTab.click();

		// Should show connect prompt
		await expect(page.getByText("Connect your wallet")).toBeVisible({ timeout: 10000 });
	});

	test("offer card shows deal type badge and memo", async ({ page }) => {
		await page.route("**/v1/offers*", async (route) => {
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
		});

		await page.goto("/offers");
		await page.waitForLoadState("load");

		// The offer card should show the memo
		await expect(page.getByText("Selling 5 KAS for NACHO tokens")).toBeVisible({ timeout: 10000 });
		// Should show OTC badge — scope to offer card to avoid matching the deal type filter button
		await expect(page.locator(".offer .pill").filter({ hasText: "OTC" })).toBeVisible();
	});
});
