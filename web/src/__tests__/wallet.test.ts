import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
	detectKasware,
	connectWallet,
	signMessage,
	mockSignature,
	subscribeToWallet,
	type WalletState,
} from "../kasware";

/**
 * Helper: create a minimal KasWare mock for each test.
 * Tests can override specific methods to simulate error conditions.
 */
function createMockKasware(overrides: Record<string, any> = {}) {
	const defaults = {
		requestAccounts: vi.fn().mockResolvedValue(["kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q"]),
		getAccounts: vi.fn().mockResolvedValue(["kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q"]),
		getPublicKey: vi.fn().mockResolvedValue("a".repeat(64)),
		getBalance: vi.fn().mockResolvedValue({ confirmed: 100_000_000_000, pending: 0 }),
		getNetwork: vi.fn().mockResolvedValue("testnet-10"),
		sendKaspa: vi.fn().mockResolvedValue("tx_hash_123"),
		signMessage: vi.fn().mockResolvedValue("ab".repeat(32)),
		getVersion: vi.fn().mockResolvedValue("1.0.0"),
		on: vi.fn(),
		removeListener: vi.fn(),
		...overrides,
	};
	return defaults;
}

describe("detectKasware", () => {
	beforeEach(() => {
		(window as any).kasware = undefined;
	});

	it("returns true when KasWare is already loaded", async () => {
		(window as any).kasware = createMockKasware();
		const result = await detectKasware(100);
		expect(result).toBe(true);
	});

	it("returns false when KasWare is not installed", async () => {
		(window as any).kasware = undefined;
		const result = await detectKasware(1); // very short timeout
		expect(result).toBe(false);
	});

	it("returns true when KasWare initializes during detection window", async () => {
		// KasWare not available yet
		(window as any).kasware = undefined;

		// It will fire the initialization event after a delay
		setTimeout(() => {
			(window as any).kasware = createMockKasware();
			window.dispatchEvent(new Event("kasware#initialized"));
		}, 5);

		const result = await detectKasware(100);
		expect(result).toBe(true);
	});
});

describe("connectWallet", () => {
	beforeEach(() => {
		(window as any).kasware = createMockKasware();
	});

	it("returns address, network and balance on success", async () => {
		const result = await connectWallet();
		expect(result).toHaveProperty("address", "kaspa:qr6g5fsvq5h4c56j8w6q8w6q8w6q8w6q");
		expect(result).toHaveProperty("network", "testnet-10");
		expect(result).toHaveProperty("balance", "1000.0000");
	});

	it("throws when KasWare is not installed", async () => {
		(window as any).kasware = undefined;
		await expect(connectWallet()).rejects.toThrow("KasWare wallet not detected");
	});

	it("throws when no accounts returned", async () => {
		(window as any).kasware = createMockKasware({
			requestAccounts: vi.fn().mockResolvedValue([]),
		});
		await expect(connectWallet()).rejects.toThrow("No accounts returned");
	});

	it("throws when user rejects connection", async () => {
		(window as any).kasware = createMockKasware({
			requestAccounts: vi.fn().mockRejectedValue(new Error("User rejected")),
		});
		await expect(connectWallet()).rejects.toThrow("User rejected");
	});

	it("handles getNetwork failure gracefully", async () => {
		(window as any).kasware = createMockKasware({
			getNetwork: vi.fn().mockRejectedValue(new Error("Network error")),
		});
		const result = await connectWallet();
		// Should still return with default network and NOT throw
		expect(result.network).toBe("unknown");
		expect(result.address).toBeTruthy();
	});

	it("handles getBalance failure gracefully", async () => {
		(window as any).kasware = createMockKasware({
			getBalance: vi.fn().mockRejectedValue(new Error("Balance error")),
		});
		const result = await connectWallet();
		// Should still return with balance "0" and NOT throw
		expect(result.balance).toBe("0");
		expect(result.address).toBeTruthy();
	});
});

describe("signMessage", () => {
	beforeEach(() => {
		(window as any).kasware = createMockKasware();
	});

	it("returns a signature on success", async () => {
		const sig = await signMessage("test message", "schnorr");
		expect(sig).toBe("ab".repeat(32));
	});

	it("throws when KasWare is not installed", async () => {
		(window as any).kasware = undefined;
		await expect(signMessage("test", "schnorr")).rejects.toThrow("KasWare wallet not detected");
	});

	it("throws when user rejects signing", async () => {
		(window as any).kasware = createMockKasware({
			signMessage: vi.fn().mockRejectedValue(new Error("User rejected")),
		});
		await expect(signMessage("test", "schnorr")).rejects.toThrow("User rejected");
	});

	it("calls signMessage with schnorr type by default", async () => {
		const mockKasware = createMockKasware();
		(window as any).kasware = mockKasware;
		await signMessage("hello");
		expect(mockKasware.signMessage).toHaveBeenCalledWith("hello", "schnorr");
	});
});

describe("mockSignature", () => {
	it("returns a deterministic hex string", () => {
		const sig1 = mockSignature("test");
		const sig2 = mockSignature("test");
		expect(sig1).toBe(sig2); // deterministic
	});

	it("returns different values for different messages", () => {
		const sig1 = mockSignature("message A");
		const sig2 = mockSignature("message B");
		expect(sig1).not.toBe(sig2);
	});

	it("always produces a 130-char hex string", () => {
		const sig = mockSignature("hello");
		expect(sig).toMatch(/^[0-9a-f]{130}$/);
	});
});

describe("subscribeToWallet", () => {
	let mockKasware: ReturnType<typeof createMockKasware>;
	let stateChanges: any[];

	beforeEach(() => {
		mockKasware = createMockKasware();
		(window as any).kasware = mockKasware;
		stateChanges = [];
	});

	it("registers event listeners on mount", () => {
		subscribeToWallet((s) => stateChanges.push(s));
		expect(mockKasware.on).toHaveBeenCalledWith("accountsChanged", expect.any(Function));
		expect(mockKasware.on).toHaveBeenCalledWith("networkChanged", expect.any(Function));
		expect(mockKasware.on).toHaveBeenCalledWith("disconnect", expect.any(Function));
	});

	it("removes listeners on unsubscribe", () => {
		const unsubscribe = subscribeToWallet(() => {});
		unsubscribe();
		expect(mockKasware.removeListener).toHaveBeenCalledWith("accountsChanged", expect.any(Function));
		expect(mockKasware.removeListener).toHaveBeenCalledWith("networkChanged", expect.any(Function));
		expect(mockKasware.removeListener).toHaveBeenCalledWith("disconnect", expect.any(Function));
	});

	it("disconnects on accountsChanged with empty array", () => {
		// In production, onStateChange is setState from React's useState,
		// which handles function updaters. Simulate that here.
		let currentState: any = { connected: true, address: "kaspa:test", network: "testnet-10" };
		const setState = (arg: any) => {
			if (typeof arg === "function") {
				currentState = arg(currentState);
			} else {
				currentState = arg;
			}
			stateChanges.push(currentState);
		};
		subscribeToWallet(setState);
		const handler = mockKasware.on.mock.calls.find(
			([event]) => event === "accountsChanged",
		)?.[1];
		handler([]);
		const last = stateChanges[stateChanges.length - 1];
		expect(last.connected).toBe(false);
		expect(last.address).toBeNull();
	});

	it("updates address on accountsChanged with new account", () => {
		let currentState: any = { connected: true, address: "kaspa:old", network: "testnet-10" };
		const setState = (arg: any) => {
			if (typeof arg === "function") {
				currentState = arg(currentState);
			} else {
				currentState = arg;
			}
			stateChanges.push(currentState);
		};
		subscribeToWallet(setState);
		const handler = mockKasware.on.mock.calls.find(
			([event]) => event === "accountsChanged",
		)?.[1];
		handler(["kaspa:newaddress123456789"]);
		const last = stateChanges[stateChanges.length - 1];
		expect(last.address).toBe("kaspa:newaddress123456789");
		// Should not disconnect — only the address should update
		expect(last.connected).toBe(true);
	});

	it("preserves connected state on networkChanged", () => {
		let currentState: any = { connected: true, address: "kaspa:test", network: "testnet-10" };
		const setState = (arg: any) => {
			if (typeof arg === "function") {
				currentState = arg(currentState);
			} else {
				currentState = arg;
			}
			stateChanges.push(currentState);
		};
		subscribeToWallet(setState);
		const handler = mockKasware.on.mock.calls.find(
			([event]) => event === "networkChanged",
		)?.[1];
		handler("testnet-10");
		const last = stateChanges[stateChanges.length - 1];
		expect(last.network).toBe("testnet-10");
		expect(last.connected).toBe(true); // should NOT reset connected
	});

	it("disconnects on disconnect event", () => {
		subscribeToWallet((s) => stateChanges.push(s));
		const handler = mockKasware.on.mock.calls.find(
			([event]) => event === "disconnect",
		)?.[1];
		handler();
		const last = stateChanges[stateChanges.length - 1];
		expect(last.connected).toBe(false);
		expect(last.address).toBeNull();
	});

	it("returns noop when KasWare is undefined", () => {
		(window as any).kasware = undefined;
		const unsubscribe = subscribeToWallet(() => {});
		expect(unsubscribe).toBeInstanceOf(Function);
		unsubscribe(); // should not throw
	});
});
