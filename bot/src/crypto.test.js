import { describe, it, before, after } from "node:test";
import assert from "node:assert";

describe("crypto", () => {
	let originalKey;
	let keyBytes;

	before(() => {
		originalKey = process.env.BOT_ENCRYPTION_KEY;
		// "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" = 32 bytes in base64
		process.env.BOT_ENCRYPTION_KEY = "YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWE=";
	});

	after(() => {
		process.env.BOT_ENCRYPTION_KEY = originalKey;
	});

	it("should export encrypt and decrypt functions", async () => {
		const mod = await import("./crypto.js");
		assert.equal(typeof mod.encrypt, "function");
		assert.equal(typeof mod.decrypt, "function");
		assert.equal(typeof mod.getEncryptionKey, "function");
		assert.equal(typeof mod.generateKey, "function");
	});

	it("should encrypt and decrypt a string", async () => {
		const mod = await import("./crypto.js");
		const key = mod.getEncryptionKey();
		const plaintext = "kaspa:testaddress123456789";

		const encrypted = mod.encrypt(plaintext, key);
		assert.notEqual(encrypted, plaintext);
		assert.ok(encrypted.includes(":")); // iv:tag:ciphertext format

		const decrypted = mod.decrypt(encrypted, key);
		assert.equal(decrypted, plaintext);
	});

	it("should return plaintext when no key (dev mode)", async () => {
		const mod = await import("./crypto.js");
		const text = "kaspa:devmode";
		assert.equal(mod.encrypt(text, null), text);
		assert.equal(mod.decrypt(text, null), text);
	});

	it("should return plaintext when decoding non-encrypted text", async () => {
		const mod = await import("./crypto.js");
		const key = mod.getEncryptionKey();
		assert.equal(mod.decrypt("plaintext", key), "plaintext");
	});

	it("should decode malformed encrypted data as plaintext", async () => {
		const mod = await import("./crypto.js");
		const key = mod.getEncryptionKey();
		assert.equal(mod.decrypt("only:two:parts:here", key), "only:two:parts:here");
	});

	it("should produce different ciphertexts for same plaintext (different IV)", async () => {
		const mod = await import("./crypto.js");
		const key = mod.getEncryptionKey();
		const a = mod.encrypt("same text", key);
		const b = mod.encrypt("same text", key);
		assert.notEqual(a, b);
	});

	it("getEncryptionKey should return null when env var not set", async () => {
		const mod = await import("./crypto.js");
		const prev = process.env.BOT_ENCRYPTION_KEY;
		delete process.env.BOT_ENCRYPTION_KEY;
		try {
			assert.equal(mod.getEncryptionKey(), null);
		} finally {
			process.env.BOT_ENCRYPTION_KEY = prev;
		}
	});

	it("getEncryptionKey should throw for wrong-length key", async () => {
		const mod = await import("./crypto.js");
		const prev = process.env.BOT_ENCRYPTION_KEY;
		process.env.BOT_ENCRYPTION_KEY = Buffer.from("tooshort").toString("base64");
		try {
			assert.throws(() => mod.getEncryptionKey(), /must decode to 32 bytes/);
		} finally {
			process.env.BOT_ENCRYPTION_KEY = prev;
		}
	});

	it("generateKey should produce a base64 32-byte key", async () => {
		const mod = await import("./crypto.js");
		const key = mod.generateKey();
		const bytes = Buffer.from(key, "base64");
		assert.equal(bytes.length, 32);
	});

	it("round-trip with generated key", async () => {
		const mod = await import("./crypto.js");
		const key = Buffer.from(mod.generateKey(), "base64");
		const original = "kaspa:qzyqpzry9x8gf2tvdw0s3jn54khce6mua7l";

		const encrypted = mod.encrypt(original, key);
		const decrypted = mod.decrypt(encrypted, key);

		assert.equal(decrypted, original);
	});
});
