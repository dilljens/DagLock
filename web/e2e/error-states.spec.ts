import { test, expect } from "../e2e/fixtures";

test.describe("Error States", () => {
	test("escrow lookup shows the lookup form", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.getByText("Lookup").click();

		const input = page.locator('input[placeholder="escrow id (esc_...)"]');
		await expect(input).toBeVisible({ timeout: 5000 });
	});

	test("health check failure is handled gracefully", async ({ page }) => {
		await page.route("**/v1/health", async (route) => {
			await route.fulfill({
				status: 500,
				contentType: "application/json",
				body: JSON.stringify({ error: "internal server error" }),
			});
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		await expect(page.locator(".dashboard-hero")).toBeVisible({ timeout: 5000 });
	});

	test("network failure shows error state gracefully", async ({ page }) => {
		await page.route("**/v1/stats", async (route) => {
			await route.abort("connectionrefused");
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		await expect(page.locator(".dashboard-hero")).toBeVisible({ timeout: 5000 });
	});
});
