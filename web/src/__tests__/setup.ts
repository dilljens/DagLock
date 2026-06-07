import "@testing-library/jest-dom";
import { vi, beforeEach } from "vitest";

beforeEach(() => {
	// Mock window.kasware for all tests
	vi.stubGlobal("kasware", {
		requestAccounts: vi.fn().mockResolvedValue(["kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q"]),
		getAccounts: vi.fn().mockResolvedValue(["kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q8w6q8w6q"]),
		getPublicKey: vi.fn().mockResolvedValue("a".repeat(64)),
		getBalance: vi.fn().mockResolvedValue({ confirmed: 100_000_000_000, pending: 0 }),
		getNetwork: vi.fn().mockResolvedValue("testnet-12"),
		sendKaspa: vi.fn().mockResolvedValue("tx_hash_123"),
		signMessage: vi.fn().mockResolvedValue("ab".repeat(32)),
		getVersion: vi.fn().mockResolvedValue("1.0.0"),
		on: vi.fn(),
		removeListener: vi.fn(),
	});
});
