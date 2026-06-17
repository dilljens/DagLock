import { describe, it, before, after, mock } from "node:test";
import assert from "node:assert";

describe("ApiClient", () => {
	let originalFetch;
	const BASE = "http://test:8543";

	// Shared mock response factory
	function mockResponse(data, status = 200, statusText = "OK") {
		return Promise.resolve({
			ok: status >= 200 && status < 300,
			status,
			statusText,
			json: () => Promise.resolve(data),
		});
	}

	before(() => {
		originalFetch = global.fetch;
	});

	after(() => {
		global.fetch = originalFetch;
	});

	it("should export ApiClient class", async () => {
		const mod = await import("./api.js");
		assert.equal(typeof mod.ApiClient, "function");
	});

	it("should GET an escrow by id", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ id: "esc_123", status: "active" }),
		);

		const result = await client.getEscrow("esc_123");
		assert.equal(result.id, "esc_123");
		assert.equal(result.status, "active");

		const callUrl = global.fetch.mock.calls[0].arguments[0];
		assert.ok(callUrl.includes("/v1/escrows/esc_123"));
	});

	it("should POST to create an escrow", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ id: "esc_new", status: "pending_confirmation" }),
		);

		const result = await client.createEscrow({
			lock_tx_id: "tx_abc",
			amount_sompi: 100000000,
		});

		assert.equal(result.id, "esc_new");

		const callArgs = global.fetch.mock.calls[0].arguments;
		assert.ok(callArgs[0].includes("/v1/escrows"));
		assert.equal(callArgs[1].method, "POST");
	});

	it("should include auth headers when provided", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() => mockResponse({ id: "vault_1", status: "locked" }));

		await client.createVault(
			{ amount_sompi: 100000000 },
			{
				address: "kaspa:alice",
				signature: "signed",
				message: "create:vault",
			},
		);

		const headers = global.fetch.mock.calls[0].arguments[1].headers;
		assert.equal(headers["X-Daglock-Address"], "kaspa:alice");
		assert.equal(headers["X-Daglock-Signature"], "signed");
		assert.equal(headers["X-Daglock-Message"], "create:vault");
	});

	it("should list escrows with query params", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ escrows: [{ id: "esc_1" }] }),
		);

		await client.listEscrows("kaspa:bob", { role: "buyer", status: "active", limit: 10, offset: 0 });

		const callUrl = global.fetch.mock.calls[0].arguments[0];
		assert.ok(callUrl.includes("address=kaspa%3Abob"));
		assert.ok(callUrl.includes("role=buyer"));
		assert.ok(callUrl.includes("status=active"));
		assert.ok(callUrl.includes("limit=10"));
	});

	it("should throw on non-2xx response", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse(
				{ error: { message: "Escrow not found" } },
				404,
				"Not Found",
			),
		);

		await assert.rejects(
			() => client.getEscrow("esc_missing"),
			/Escrow not found/,
		);
	});

	it("should retry on GET 5xx up to MAX_RETRIES times", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		let callCount = 0;
		global.fetch = mock.fn(() => {
			callCount++;
			return mockResponse({ error: { message: "Server error" } }, 503);
		});

		await assert.rejects(() => client.listEscrows("kaspa:x"));
		// 1 initial + 3 retries = 4 total
		assert.ok(callCount >= 3, `Expected >=3 calls, got ${callCount}`);
	});

	it("should NOT retry on POST errors", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		let callCount = 0;
		global.fetch = mock.fn(() => {
			callCount++;
			return mockResponse({ error: { message: "Server error" } }, 503);
		});

		await assert.rejects(() =>
			client.createEscrow({ lock_tx_id: "tx_test" }),
		);

		// POST should fail immediately, no retry
		assert.equal(callCount, 1);
	});

	it("should get reputation", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ score: 4.5, trade_count: 10 }),
		);

		const result = await client.getReputation("kaspa:trader");
		assert.equal(result.score, 4.5);
	});

	it("should list offers", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ offers: [{ id: "off_1" }] }),
		);

		const result = await client.listOffers({ asset: "KAS", side: "buy" });
		assert.ok(result.offers.length > 0);
	});

	it("should get health", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ status: "ok", db_connected: true }),
		);

		const result = await client.getHealth();
		assert.equal(result.status, "ok");
	});

	it("should accept offer with counterparty address", async () => {
		const mod = await import("./api.js");
		const client = new mod.ApiClient(BASE);

		global.fetch = mock.fn(() =>
			mockResponse({ id: "off_1", status: "accepted" }),
		);

		const result = await client.acceptOffer("off_1", "kaspa:buyer");
		assert.equal(result.id, "off_1");

		const body = JSON.parse(global.fetch.mock.calls[0].arguments[1].body);
		assert.equal(body.counterparty_address, "kaspa:buyer");
	});
});
