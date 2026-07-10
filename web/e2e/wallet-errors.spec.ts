import { test, expect } from "./fixtures";
import { TEST_ADDRESS } from "./helpers/kasware";

test.describe("Wallet Error Handling", () => {
	test("shows error when requestAccounts returns empty", async ({ page }) => {
		// Override the mock to return empty accounts
		await page.addInitScript(() => {
			(window as any).kasware = {
				...((window as any).kasware || {}),
				requestAccounts: async () => [],
			};
		});

		await page.goto("/");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet-error")).toBeVisible({ timeout: 5000 });
	});

	test("shows error when KasWare throws during connect", async ({ page }) => {
		await page.addInitScript(() => {
			(window as any).kasware = {
				...((window as any).kasware || {}),
				requestAccounts: async () => { throw new Error("User rejected connection"); },
			};
		});

		await page.goto("/");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1500);

		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet-error")).toBeVisible({ timeout: 5000 });
	});

	test("still connects when getNetwork fails", async ({ page }) => {
		await page.addInitScript(() => {
			(window as any).kasware = {
				...((window as any).kasware || {}),
				getNetwork: async () => { throw new Error("Network error"); },
			};
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });
	});

	test("still connects when getBalance fails", async ({ page }) => {
		await page.addInitScript(() => {
			(window as any).kasware = {
				...((window as any).kasware || {}),
				getBalance: async () => { throw new Error("Balance error"); },
			};
		});

		await page.goto("/");
		await page.waitForLoadState("load");

		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });
	});

	test("shows disconnected state on disconnect event", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");
		await page.waitForTimeout(1000);

		// Connect first
		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });

		// Simulate KasWare disconnect via the exposed event system
		await page.evaluate(() => {
			const k = (window as any).kasware;
			if (k && k._fireEvent) k._fireEvent("disconnect");
		});

		// Should show connect button again
		await expect(page.locator(".sidebar-connect")).toBeVisible({ timeout: 5000 });
	});

	test("updates address on account change", async ({ page }) => {
		await page.goto("/");
		await page.waitForLoadState("load");

		// Connect
		await page.locator(".sidebar-connect").click();
		await expect(page.locator(".sidebar-wallet")).toBeVisible({ timeout: 5000 });

		// Simulate account change to different address
		await page.evaluate(() => {
			const k = (window as any).kasware;
			if (k && k._fireEvent) k._fireEvent("accountsChanged", ["kaspa:newaccountaddress1234567890abcdef"]);
		});

		// Address should update (but stay connected)
		await expect(page.locator(".sidebar-wallet-addr")).toContainText("kaspa:newaccount", { timeout: 5000 });
	});
});
