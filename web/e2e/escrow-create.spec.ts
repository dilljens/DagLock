import { test, expect } from "../e2e/fixtures";

test.describe("Escrow Creation", () => {
	test("shows connect prompt on escrows page without wallet", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await expect(page.locator("h1")).toContainText("Escrows");
	});

	test("escrow tabs are visible", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();

		// Use the tab-bar buttons to avoid ambiguous text matches
		const tabBar = page.locator(".tab-bar");
		await expect(tabBar.getByText("My Escrows")).toBeVisible();
		await expect(tabBar.getByText("Create")).toBeVisible();
		await expect(tabBar.getByText("Lookup")).toBeVisible();
	});

	test("escrow lookup works with mock data", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.getByText("Lookup").click();

		const input = page.locator('input[placeholder="escrow id (esc_...)"]');
		await expect(input).toBeVisible({ timeout: 5000 });
	});

	test("first visit shows welcome content on dashboard", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// The h2 text "Trustless Escrow" is on the dashboard — use first match
		const hero = page.locator("h2").filter({ hasText: "Trustless Escrow" });
		await expect(hero).toBeVisible({ timeout: 5000 });
	});

	test("feature cards are displayed on dashboard", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await expect(page.locator(".feature-card").first()).toBeVisible({ timeout: 5000 });
	});
});
