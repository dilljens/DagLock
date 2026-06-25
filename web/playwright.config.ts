import { defineConfig, devices } from "@playwright/test";

// Port convention: each project must use a unique dev server port.
// DagLock=5173, TalkEdit=5175, ScriptureExplorer=5176, Inklomancer=5183.
// Never share port 5173 — update vite.config.ts + this file together.
const PORT = 5173;

export default defineConfig({
	testDir: "./e2e",
	fullyParallel: false,
	retries: 0,
	workers: 1,
	reporter: "list",
	use: {
		baseURL: `http://localhost:${PORT}`,
		trace: "on-first-retry",
	},
	projects: [
		{
			name: "manual-wallet",
			testMatch: "**/*.spec.ts",
			testIgnore: "**/kasware-wallet.spec.ts",
			use: { ...devices["Desktop Chrome"] },
		},
		{
			name: "kasware-wallet",
			testMatch: "**/kasware-wallet.spec.ts",
			use: {
				...devices["Desktop Chrome"],
				kaswareMock: {},
			},
		},
		{
			name: "mobile",
			testMatch: "**/mobile.spec.ts",
			use: {
				browserName: "chromium",
				viewport: { width: 375, height: 812 },
			},
		},
	],
});
