// AES-256-GCM encryption for user data at rest.
// Key is a base64-encoded 32-byte value from BOT_ENCRYPTION_KEY env var.

import crypto from "node:crypto";

const ALGORITHM = "aes-256-gcm";

export function getEncryptionKey() {
	const key = process.env.BOT_ENCRYPTION_KEY;
	if (!key) return null;
	const bytes = Buffer.from(key, "base64");
	if (bytes.length !== 32) {
		throw new Error(
			`BOT_ENCRYPTION_KEY must decode to 32 bytes (got ${bytes.length})`,
		);
	}
	return bytes;
}

export function encrypt(text, keyBytes) {
	if (!keyBytes) return text; // dev mode — store as plaintext
	const iv = crypto.randomBytes(12); // 96-bit IV for GCM
	const cipher = crypto.createCipheriv(ALGORITHM, keyBytes, iv);
	let encrypted = cipher.update(text, "utf-8", "hex");
	encrypted += cipher.final("hex");
	const tag = cipher.getAuthTag().toString("hex");
	return iv.toString("hex") + ":" + tag + ":" + encrypted;
}

export function decrypt(encoded, keyBytes) {
	if (!keyBytes || !encoded.includes(":")) return encoded; // plaintext fallback
	const parts = encoded.split(":");
	if (parts.length !== 3) return encoded;
	const iv = Buffer.from(parts[0], "hex");
	const tag = Buffer.from(parts[1], "hex");
	const encrypted = parts[2];
	const decipher = crypto.createDecipheriv(ALGORITHM, keyBytes, iv);
	decipher.setAuthTag(tag);
	let decrypted = decipher.update(encrypted, "hex", "utf-8");
	decrypted += decipher.final("utf-8");
	return decrypted;
}

export function generateKey() {
	return crypto.randomBytes(32).toString("base64");
}
