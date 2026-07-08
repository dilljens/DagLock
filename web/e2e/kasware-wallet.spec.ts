import { test, expect } from "../e2e/fixtures";
import { TEST_ADDRESS } from "./helpers/kasware";

test.describe("KasWare Mock Wallet", () => {
	test("sidebar shows Connect Wallet when KasWare detected", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await expect(page.locator(".sidebar-connect")).toContainText("Connect Wallet", { timeout: 5000 });
	});

	test("sidebar connect button is visible (not Install KasWare)", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await expect(page.locator(".sidebar-connect")).not.toContainText("Install KasWare");
	});

	test("sidebar shows wallet info after connecting", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await page.locator(".sidebar-connect").click();

		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });
		await expect(page.locator(".sidebar-wallet-addr")).toContainText(TEST_ADDRESS.slice(0, 10));
	});

	test("dashboard shows connected state after connect", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();

		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });
	});

	test("escrows page shows create tab with connected wallet", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();
		await page.waitForTimeout(500);

		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.waitForTimeout(500);

		await page.locator(".tab-bar").getByText("Create").click();
		await expect(page.getByText("Amount (KAS)")).toBeVisible({ timeout: 5000 });
	});

	test("escrow create form has required fields", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.locator(".tab-bar").getByText("Create").click();

		await expect(page.locator('input[placeholder="100"]')).toBeVisible({ timeout: 5000 });
		await expect(page.getByText("Dispute resolution")).toBeVisible({ timeout: 5000 });
		await expect(page.locator('button.primary').filter({ hasText: "Create escrow" })).toBeVisible({ timeout: 5000 });
	});

	test("dispute resolution dropdown has standard, mediator, jury options", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();
		await page.locator(".sidebar-link").filter({ hasText: "Escrows" }).click();
		await page.locator(".tab-bar").getByText("Create").click();

		// Select options aren't visible in DOM — check via option values
		const select = page.locator("select");
		await expect(select).toBeVisible({ timeout: 5000 });
		const optionTexts = await select.locator("option").allInnerTexts();
		expect(optionTexts).toContain("Standard (timeout refund)");
		expect(optionTexts).toContain("Specific mediator");
		expect(optionTexts).toContain("Jury (community vote)");
	});

	test("sidebar shows testnet network badge after connecting", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();

		await expect(page.locator(".sidebar-network")).toBeVisible({ timeout: 5000 });
	});
});
