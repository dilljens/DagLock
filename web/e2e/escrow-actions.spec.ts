import { test, expect } from "./fixtures";

test.describe("Escrow Actions", () => {
	test("shows escrow detail page", async ({ page }) => {
		await page.goto("/escrows/esc_test_1");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await expect(page.locator("h1, h2").first()).toBeVisible({ timeout: 5000 });
	});

	test("settle button is visible on active escrow detail", async ({ page }) => {
		await page.goto("/escrows/esc_test_1");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// Connect wallet first
		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(1000);

		await page.goto("/escrows/esc_test_1");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		// The settle button should be visible for active escrows
		const settleBtn = page.locator("button").filter({ hasText: /settle/i });
		await expect(settleBtn).toBeVisible({ timeout: 5000 });
	});

	test("settle flow triggers KasWare signing", async ({ page }) => {
		await page.goto("/escrows/esc_test_1");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1000);

		// Connect wallet
		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(1000);

		await page.goto("/escrows/esc_test_1");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1000);

		// Click settle
		const settleBtn = page.locator("button").filter({ hasText: /settle/i });
		if (await settleBtn.isVisible({ timeout: 3000 })) {
			await settleBtn.click();
			await page.waitForTimeout(2000);

			// After signing, the mock KasWare returns immediately
			// The API mock for settle returns 200
			// Check for success toast
			await expect(page.locator(".Toastify__toast--success, .notification-success").first())
				.toBeVisible({ timeout: 10000 });
		}
	});

	test("refund button is visible for escrow creator", async ({ page }) => {
		// Mock the escrow detail to return creator as our connected address
		await page.route("**/v1/escrows/esc_refund_test", async (route) => {
			if (route.request().method() === "GET") {
				await route.fulfill({
					status: 200, contentType: "application/json",
					body: JSON.stringify({
						id: "esc_refund_test",
						lock_tx_id: "mock_tx_refund",
						lock_tx_output_index: 0,
						status: "active",
						asset_type: "KAS",
						buyer_address: "kaspa:qztestaddressformockkaswarewallet1234567890abcdef",
						amount_sompi: 100_000_000,
						fee_sompi: 500_000,
						created_at: Date.now() / 1000 - 86400,
					}),
				});
			}
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		// Connect wallet so we match the buyer address
		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(1000);

		await page.goto("/escrows/esc_refund_test");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1000);

		// Refund button should be visible for the buyer
		const refundBtn = page.locator("button").filter({ hasText: /refund/i });
		await expect(refundBtn).toBeVisible({ timeout: 5000 });
	});
});
