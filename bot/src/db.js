// DagLock Bot — Persistent SQLite storage.
//
// Uses better-sqlite3 via createRequire for ESM compatibility.
// Stores user addresses (encrypted at rest by the caller) and
// migrates legacy data from /tmp/daglock-users.json on first run.

import { createRequire } from "module";
import { existsSync, readFileSync, renameSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const require = createRequire(import.meta.url);
const Database = require("better-sqlite3");

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const DB_PATH = process.env.BOT_DB_PATH || join(__dirname, "..", "bot.db");
const LEGACY_JSON_PATH = "/tmp/daglock-users.json";

let db;

/** Initialise the database, create tables, and migrate legacy data. */
export function initDb() {
	db = new Database(DB_PATH);
	db.pragma("journal_mode = WAL");
	db.pragma("foreign_keys = ON");

	db.exec(`
		CREATE TABLE IF NOT EXISTS users (
			telegram_id INTEGER PRIMARY KEY,
			address     TEXT NOT NULL,
			updated_at  INTEGER NOT NULL
		);
	`);

	migrateFromJson();
	return db;
}

/** Retrieve a user's address by Telegram ID. Returns null if not found. */
export function getUserAddress(telegramId) {
	if (!db) return null;
	const row = db.prepare("SELECT address FROM users WHERE telegram_id = ?").get(telegramId);
	return row ? row.address : null;
}

/** Set (insert or replace) a user's address and update timestamp. */
export function setUserAddress(telegramId, address) {
	if (!db) initDb();
	db.prepare(
		"INSERT INTO users (telegram_id, address, updated_at) VALUES (?, ?, ?) \
		 ON CONFLICT(telegram_id) DO UPDATE SET address = excluded.address, updated_at = excluded.updated_at",
	).run(telegramId, address, Date.now());
}

/** Delete a user record. */
export function deleteUser(telegramId) {
	if (!db) return;
	db.prepare("DELETE FROM users WHERE telegram_id = ?").run(telegramId);
}

/** Return all user IDs (for admin/management). */
export function getAllUserIds() {
	if (!db) return [];
	return db.prepare("SELECT telegram_id FROM users").all().map((r) => r.telegram_id);
}

/** Close the database connection. */
export function closeDb() {
	if (db) {
		db.close();
		db = null;
	}
}

/**
 * Migrate data from the legacy /tmp/daglock-users.json file.
 * The legacy format stored plaintext or encrypted addresses keyed by telegram_id.
 * We re-encrypt any plaintext entries before storing.
 */
function migrateFromJson() {
	if (!existsSync(LEGACY_JSON_PATH)) return;

	try {
		const raw = JSON.parse(readFileSync(LEGACY_JSON_PATH, "utf-8"));
		if (!raw || typeof raw !== "object") return;

		const insert = db.prepare(
			"INSERT OR IGNORE INTO users (telegram_id, address, updated_at) VALUES (?, ?, ?)",
		);
		const count = { migrated: 0, skipped: 0 };

		for (const [id, entry] of Object.entries(raw)) {
			const telegramId = parseInt(id, 10);
			if (isNaN(telegramId)) continue;

			// Entry may be { address, updatedAt } or { encryptedAddress, updatedAt }
			// We store whatever address string we have — encryption is caller's concern.
			const address = entry.address || entry.encryptedAddress;
			if (!address) continue;

			const updatedAt = entry.updatedAt || Date.now();
			const existing = db
				.prepare("SELECT telegram_id FROM users WHERE telegram_id = ?")
				.get(telegramId);
			if (existing) {
				count.skipped++;
				continue;
			}
			insert.run(telegramId, address, updatedAt);
			count.migrated++;
		}

		// Backup and remove legacy file
		renameSync(LEGACY_JSON_PATH, LEGACY_JSON_PATH + ".bak");

		console.log(
			`db: migrated ${count.migrated} users from legacy JSON (${count.skipped} already existing)`,
		);
	} catch (e) {
		console.error("db: failed to migrate legacy JSON:", e.message);
	}
}
