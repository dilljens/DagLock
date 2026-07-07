import { test as base } from "@playwright/test";
import { mockKasware, type KaswareProvider } from "./helpers/kasware";
import { setupApiMocks } from "./helpers/api";

type Fixtures = {
	kaswareMock: Partial<KaswareProvider> | undefined;
};

export const test = base.extend<Fixtures>({
	kaswareMock: [undefined, { option: true }],

	page: async ({ page, kaswareMock }, use) => {
		await setupApiMocks(page);
		if (kaswareMock) {
			await mockKasware(page, kaswareMock);
		}
		// Dismiss onboarding overlay on first page load
		await page.addInitScript(() => {
			localStorage.setItem("daglock_onboarded", "true");
		});
		await use(page);
	},
});

export { expect } from "@playwright/test";
