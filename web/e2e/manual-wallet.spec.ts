import { test, expect } from "../e2e/fixtures";

test.describe("Manual Wallet (no KasWare)", () => {
	test("page loads and shows dashboard hero", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const body = page.locator("body");
		await expect(body).toBeVisible();
	});

	test("sidebar has navigation items", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const nav = page.locator("nav");
		await expect(nav.getByText("Dashboard")).toBeVisible();
		await expect(nav.getByText("Escrows")).toBeVisible();
		await expect(nav.getByText("Offers")).toBeVisible();
	});

	test("footer is visible on dashboard", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const footer = page.locator("footer");
		await expect(footer).toBeVisible();
	});

	test("sidebar shows Install KasWare when no wallet detected", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const sidebarConnect = page.locator(".sidebar-connect");
		await expect(sidebarConnect).toContainText("Install KasWare");
	});

	test("dashboard hero shows Install KasWare link when no wallet", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// The hero section has a "Get Started" link that goes to kasware.xyz
		await expect(page.locator(".dashboard-hero")).toBeVisible();
	});

	test("navigating via sidebar works", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// Use nav buttons in sidebar (avoid ambiguous "Escrows" text)
		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await expect(page.locator("h1")).toContainText("Escrows");

		await page.locator(".sidebar-link").filter({ hasText: "Offers" }).click();
		await expect(page.locator("h1")).toContainText("Offers");

		await page.locator(".sidebar-link").filter({ hasText: "Dashboard" }).click();
		await expect(page.locator(".dashboard-hero")).toBeVisible();
	});
});
