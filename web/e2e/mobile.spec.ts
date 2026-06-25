import { test, expect } from "../e2e/fixtures";

test.describe("Mobile Viewport", () => {
	test.beforeEach(async ({ page }) => {
		await page.setViewportSize({ width: 375, height: 812 });
	});

	test("page loads without overflow on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const bodyWidth = await page.evaluate(() => document.body.scrollWidth);
		expect(bodyWidth).toBeLessThanOrEqual(400);
	});

	test("sidebar is not visible on mobile by default", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// On mobile the sidebar renders but is off-screen (transform)
		const sidebar = page.locator(".sidebar");
		await expect(sidebar).toBeVisible();
		// It should NOT have the sidebar--open class
		await expect(page.locator(".sidebar--open")).not.toBeVisible();
	});

	test("hamburger menu is visible on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const hamburger = page.locator(".hamburger");
		await expect(hamburger).toBeVisible({ timeout: 5000 });
	});

	test("hamburger menu opens sidebar on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".hamburger").click();
		// The .sidebar-overlay should appear when sidebar is open
		await expect(page.locator(".sidebar-overlay")).toBeVisible({ timeout: 5000 });
	});

	test("sidebar overlay is visible when sidebar open on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".hamburger").click();
		await expect(page.locator(".sidebar-overlay")).toBeVisible({ timeout: 5000 });
	});

	test("clicking sidebar nav on mobile closes sidebar", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".hamburger").click();
		await page.waitForTimeout(300);
		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();

		await expect(page.locator(".sidebar-overlay")).not.toBeVisible();
	});

	test("dashboard hero is readable on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await expect(page.locator(".dashboard-hero")).toBeVisible({ timeout: 5000 });
	});

	test("escrow lookup form is usable on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// Open sidebar via hamburger, then navigate (sidebar closes on nav)
		await page.locator(".hamburger").click();
		await page.waitForTimeout(300);
		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.waitForTimeout(500);

		// Click Lookup in the tab-bar
		await page.locator(".tab-bar").getByText("Lookup").click();
		await page.waitForTimeout(500);

		const input = page.locator('input[placeholder="escrow id (esc_...)"]');
		await expect(input).toBeVisible({ timeout: 5000 });
		await input.fill("esc_test");

		const fetchBtn = page.locator('button[type="submit"]').filter({ hasText: "Fetch" });
		await expect(fetchBtn).toBeVisible();
		await expect(fetchBtn).toBeEnabled();
	});

	test("main content has no horizontal scroll on mobile", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		const overflowX = await page.evaluate(() => {
			const main = document.querySelector(".main-content");
			if (!main) return "not-found";
			return window.getComputedStyle(main).overflowX;
		});

		expect(overflowX).not.toBe("scroll");
		expect(overflowX).not.toBe("auto");
	});
});
