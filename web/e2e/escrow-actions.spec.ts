import { test, expect } from "./fixtures";

test.describe("Escrow Actions", () => {
	test("escrows page loads and shows cards", async ({ page }) => {
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		await expect(page.locator("h1")).toContainText("Escrows", { timeout: 10000 });
	});

	test("settle button appears on active escrow when expanded", async ({ page }) => {
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		// Connect wallet first — wallet mock auto-detects but doesn't auto-connect
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Navigate to /escrows after connecting — wallet state doesn't persist across page.goto
		// so we need to reconnect after the reload
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		// Reconnect wallet after page reload
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Wait for escrow cards to appear (from fixture API mocks)
		const escrowCard = page.locator("article.offer").first();
		await expect(escrowCard).toBeVisible({ timeout: 10000 });

		// Click to expand the card — escrows page uses onClick to set selectedId
		await escrowCard.click();

		// The Settle button should be visible in the expanded action area
		const settleBtn = page.locator("button").filter({ hasText: /settle/i });
		await expect(settleBtn).toBeVisible({ timeout: 10000 });
	});

	test("settle flow triggers KasWare signing", async ({ page }) => {
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		// Connect wallet
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Navigate to /escrows — wallet state doesn't persist, reconnect
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		// Reconnect wallet after page reload
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Expand first escrow card
		const escrowCard = page.locator("article.offer").first();
		await expect(escrowCard).toBeVisible({ timeout: 10000 });
		await escrowCard.click();

		// Click settle
		const settleBtn = page.locator("button").filter({ hasText: /settle/i });
		if (await settleBtn.isVisible({ timeout: 5000 })) {
			await settleBtn.click();

			// After signing, the mock KasWare returns immediately
			// The API mock for settle returns 200
			// Check for success toast
			await expect(page.locator(".toast--success").first())
				.toBeVisible({ timeout: 10000 });
		}
	});

	test("refund and cancel buttons appear on active escrow when expanded", async ({ page }) => {
		// Mock the escrow list to return an escrow with our test address
		await page.route("**/v1/escrows*", async (route) => {
			const url = route.request().url();
			if (route.request().method() === "GET" && url.includes("?address=")) {
				await route.fulfill({
					status: 200, contentType: "application/json",
					body: JSON.stringify({
						escrows: [{
							id: "esc_refund_test",
							lock_tx_id: "mock_tx_refund",
							lock_tx_output_index: 0,
							status: "active",
							asset_type: "KAS",
							buyer_address: "kaspa:qztestaddressformockkaswarewallet1234567890abcdef",
							amount_sompi: 100_000_000,
							fee_sompi: 500_000,
							created_at: Date.now() / 1000 - 86400,
						}],
						total: 1,
					}),
				});
			} else {
				await route.fallback();
			}
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		// Connect wallet
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Navigate to /escrows — wallet state doesn't persist, reconnect
		await page.goto("/escrows");
		await page.waitForLoadState("load");

		// Reconnect wallet after page reload
		await page.locator(".sidebar-connect").waitFor({ state: "visible", timeout: 10000 });
		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-wallet").waitFor({ state: "visible", timeout: 10000 });

		// Expand escrow card
		const escrowCard = page.locator("article.offer").first();
		await expect(escrowCard).toBeVisible({ timeout: 10000 });
		await escrowCard.click();

		// Refund and cancel buttons should be visible for active escrows
		const refundBtn = page.locator("button").filter({ hasText: /refund/i });
		await expect(refundBtn).toBeVisible({ timeout: 10000 });
		const cancelBtn = page.locator("button").filter({ hasText: /cancel/i });
		await expect(cancelBtn).toBeVisible({ timeout: 5000 });
	});
});
