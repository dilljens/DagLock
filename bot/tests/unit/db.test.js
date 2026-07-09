import { describe, it, before, after } from "node:test";
import assert from "node:assert";
import { unlinkSync, existsSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

describe("Bot SQLite DB", () => {
	const DB_PATH = join(__dirname, "..", "..", "test-bot.db");

	before(() => {
		process.env.BOT_DB_PATH = DB_PATH;
		// Clean up any leftover test db
		if (existsSync(DB_PATH)) unlinkSync(DB_PATH);
	});

	after(() => {
		delete process.env.BOT_DB_PATH;
		if (existsSync(DB_PATH)) unlinkSync(DB_PATH);
	});

	it("should init DB and create tables", async () => {
		const { initDb, closeDb } = await import("../../src/db.js");
		initDb();
		closeDb();
		assert.ok(existsSync(DB_PATH), "DB file should exist");
	});

	it("should set and get a user address", async () => {
		const { initDb, setUserAddress, getUserAddress, closeDb } = await import("../../src/db.js");
		initDb();
		setUserAddress(12345, "kaspa:test:address123");
		const result = getUserAddress(12345);
		assert.equal(result, "kaspa:test:address123");
		closeDb();
	});

	it("should return null for unknown user", async () => {
		const { initDb, getUserAddress, closeDb } = await import("../../src/db.js");
		initDb();
		const result = getUserAddress(99999);
		assert.equal(result, null);
		closeDb();
	});

	it("should update existing user address", async () => {
		const { initDb, setUserAddress, getUserAddress, closeDb } = await import("../../src/db.js");
		initDb();
		setUserAddress(12345, "kaspa:first");
		setUserAddress(12345, "kaspa:updated");
		const result = getUserAddress(12345);
		assert.equal(result, "kaspa:updated");
		closeDb();
	});

	it("should delete a user", async () => {
		const { initDb, setUserAddress, getUserAddress, deleteUser, closeDb } =
			await import("../../src/db.js");
		initDb();
		setUserAddress(12345, "kaspa:delete:me");
		deleteUser(12345);
		const result = getUserAddress(12345);
		assert.equal(result, null);
		closeDb();
	});

	it("should list all user IDs", async () => {
		const { initDb, setUserAddress, getAllUserIds, closeDb } = await import("../../src/db.js");
		initDb();
		setUserAddress(1, "kaspa:user1");
		setUserAddress(2, "kaspa:user2");
		setUserAddress(3, "kaspa:user3");
		const ids = getAllUserIds();
		assert.deepEqual(ids.sort(), [1, 2, 3]);
		closeDb();
	});

	it("should migrate legacy JSON data", async () => {
		const { initDb, getUserAddress, closeDb } = await import("../../src/db.js");

		// Create a mock legacy file
		const legacyPath = "/tmp/daglock-users.json";
		const legacyData = {
			"555": { address: "kaspa:legacy:user", updatedAt: 1000000 },
			"666": { address: "kaspa:legacy:two", updatedAt: 1000001 },
		};
		writeFileSync(legacyPath, JSON.stringify(legacyData));

		initDb();

		// After init, the legacy data should be migrated
		const user1 = getUserAddress(555);
		assert.equal(user1, "kaspa:legacy:user");

		const user2 = getUserAddress(666);
		assert.equal(user2, "kaspa:legacy:two");

		// Legacy file should be backed up
		assert.ok(!existsSync(legacyPath), "legacy file should be removed");
		assert.ok(existsSync(legacyPath + ".bak"), "backup should exist");

		closeDb();
		// Clean up backup
		if (existsSync(legacyPath + ".bak")) unlinkSync(legacyPath + ".bak");
	});
});
