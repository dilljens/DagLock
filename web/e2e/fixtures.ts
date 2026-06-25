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
		await use(page);
	},
});

export { expect } from "@playwright/test";
