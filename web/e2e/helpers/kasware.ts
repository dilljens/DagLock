import type { Page } from "@playwright/test";

export interface KaswareProvider {
	requestAccounts(): Promise<string[]>;
	getAccounts(): Promise<string[]>;
	getPublicKey(): Promise<string>;
	getBalance(): Promise<{ confirmed: number; pending: number }>;
	getNetwork(): Promise<string>;
	sendKaspa(to: string, sompi: number, opts?: { feeRate?: number }): Promise<string>;
	signMessage(message: string, type?: "ecdsa" | "schnorr"): Promise<string>;
	getVersion(): Promise<string>;
	on(event: string, handler: (data?: any) => void): void;
	removeListener(event: string, handler: (data?: any) => void): void;
}

export const TEST_ADDRESS = "kaspa:qztestaddressformockkaswarewallet1234567890abcdef";
export const TEST_PUBKEY = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

export function createMockKasware(overrides?: Partial<KaswareProvider>): KaswareProvider {
	return {
		requestAccounts: async () => [TEST_ADDRESS],
		getAccounts: async () => [TEST_ADDRESS],
		getPublicKey: async () => TEST_PUBKEY,
		getBalance: async () => ({ confirmed: 500_000_000, pending: 0 }),
		getNetwork: async () => "testnet-10",
		sendKaspa: async (_to: string, _sompi: number, _opts?: { feeRate?: number }) =>
			"mock_tx_id_abcdef1234567890",
		signMessage: async (_message: string, _type?: "ecdsa" | "schnorr") =>
			"ff" + "a".repeat(126),
		getVersion: async () => "1.0.0",
		on: () => {},
		removeListener: () => {},
		...overrides,
	};
}

export async function mockKasware(page: Page, _overrides?: Partial<KaswareProvider>): Promise<void> {
	// Build the mock fully inline so async functions survive JSON serialization.
	const addr = TEST_ADDRESS;
	const pubkey = TEST_PUBKEY;
	await page.addInitScript(`(() => {
		window.kasware = {
			requestAccounts: async () => ["${addr}"],
			getAccounts: async () => ["${addr}"],
			getPublicKey: async () => "${pubkey}",
			getBalance: async () => ({ confirmed: 500000000, pending: 0 }),
		getNetwork: async () => "testnet-10",
			sendKaspa: async (_to, _sompi, _opts) => "mock_tx_id_abcdef1234567890",
			signMessage: async (_msg, _type) => "ff" + "a".repeat(126),
			getVersion: async () => "1.0.0",
			on: () => {},
			removeListener: () => {},
		};
		window.dispatchEvent(new Event("kasware#initialized"));
	})();`);
}
