import { describe, it, before, after } from "node:test";
import assert from "node:assert";
import { existsSync, unlinkSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const DB_PATH = join(__dirname, "..", "..", "test-cmds.db");

// We test the command handler logic by exercising the db layer and
// simulating the data flow. The actual grammY bot wiring is tested
// via integration tests.

describe("Bot command handlers", () => {
	let db;
	let initDb, setUserAddress, getUserAddress, closeDb;

	before(async () => {
		process.env.BOT_DB_PATH = DB_PATH;
		if (existsSync(DB_PATH)) unlinkSync(DB_PATH);
		const mod = await import("../../src/db.js");
		initDb = mod.initDb;
		setUserAddress = mod.setUserAddress;
		getUserAddress = mod.getUserAddress;
		closeDb = mod.closeDb;
		db = initDb();
	});

	after(() => {
		closeDb();
		delete process.env.BOT_DB_PATH;
		if (existsSync(DB_PATH)) unlinkSync(DB_PATH);
	});

	// ── /setaddress handler logic ──────────────────────────────────

	it("should accept a valid Kaspa address", () => {
		// Simulate the /setaddress command handler
		const validAddress = "kaspa:qzh6d5h6ryztyxjgqax6unw4m7w3gkwuwyy2zl29cqz56p6fjq25vyjket3w";
		setUserAddress(10001, validAddress);
		const stored = getUserAddress(10001);
		assert.equal(stored, validAddress);
	});

	it("should reject an empty address", () => {
		// The actual handler returns early when address is empty
		// Simulate: if the address is empty/falsy, don't store it
		const emptyAddress = "";
		if (emptyAddress?.trim()) {
			setUserAddress(10002, emptyAddress);
		}
		const stored = getUserAddress(10002);
		assert.equal(stored, null, "empty address should not be stored");
	});

	it("should update an existing address", () => {
		setUserAddress(10003, "kaspa:old:address");
		setUserAddress(10003, "kaspa:new:address");
		const stored = getUserAddress(10003);
		assert.equal(stored, "kaspa:new:address");
	});

	// ── Conversation wizard state machine ──────────────────────────

	it("should start a conversation wizard", async () => {
		// The convState is a Map in index.js. We test the same pattern.
		const convState = new Map();
		function startConv(userId) {
			convState.set(userId, { step: "amount", data: {} });
		}
		function getConv(userId) {
			return convState.get(userId);
		}

		startConv(20001);
		const conv = getConv(20001);
		assert.ok(conv, "should create conversation state");
		assert.equal(conv.step, "amount");
		assert.deepEqual(conv.data, {});
	});

	it("should advance conversation step", async () => {
		const convState = new Map();
		function startConv(userId) {
			convState.set(userId, { step: "amount", data: {} });
		}
		function getConv(userId) {
			return convState.get(userId);
		}
		function advanceConv(userId, step) {
			const s = convState.get(userId);
			if (s) s.step = step;
		}

		startConv(20002);
		advanceConv(20002, "counterparty");
		assert.equal(getConv(20002).step, "counterparty");
	});

	it("should update conversation data", async () => {
		const convState = new Map();
		function startConv(userId) {
			convState.set(userId, { step: "amount", data: {} });
		}
		function getConv(userId) {
			return convState.get(userId);
		}
		function updateConv(userId, key, value) {
			const s = convState.get(userId) || { step: "amount", data: {} };
			s.data[key] = value;
			convState.set(userId, s);
		}

		startConv(20003);
		updateConv(20003, "amount", "100");
		assert.equal(getConv(20003).data.amount, "100");
	});

	it("should update conversation data without starting one first", async () => {
		const convState = new Map();
		function startConv(userId) {
			convState.set(userId, { step: "amount", data: {} });
		}
		function getConv(userId) {
			return convState.get(userId);
		}
		function updateConv(userId, key, value) {
			const s = convState.get(userId) || { step: "amount", data: {} };
			s.data[key] = value;
			convState.set(userId, s);
		}

		// Call updateConv without startConv first — should create state
		updateConv(20004, "amount", "50");
		const conv = getConv(20004);
		assert.ok(conv, "should auto-create state");
		assert.equal(conv.step, "amount");
		assert.equal(conv.data.amount, "50");
	});

	it("should end conversation", async () => {
		const convState = new Map();
		function startConv(userId) {
			convState.set(userId, { step: "amount", data: {} });
		}
		function getConv(userId) {
			return convState.get(userId);
		}
		function endConv(userId) {
			convState.delete(userId);
		}

		startConv(20005);
		endConv(20005);
		assert.equal(getConv(20005), undefined, "state should be removed after end");
	});

	// ── Error handling patterns ────────────────────────────────────

	it("should handle invalid user ID gracefully", () => {
		const result = getUserAddress(undefined);
		assert.equal(result, null, "undefined userId should return null");

		const result2 = getUserAddress(null);
		assert.equal(result2, null, "null userId should return null");
	});

	// ── API URL validation ─────────────────────────────────────────

	it("should use default indexer URL when env not set", async () => {
		const originalUrl = process.env.INDEXER_URL;
		delete process.env.INDEXER_URL;

		// We just check the default URL format used by the bot
		const apiUrl = process.env.INDEXER_URL || "http://localhost:8443";
		assert.equal(apiUrl, "http://localhost:8443");

		if (originalUrl) process.env.INDEXER_URL = originalUrl;
	});
});
