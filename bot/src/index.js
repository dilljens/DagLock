// DagLock Telegram Bot — Trustless escrow from your chat.
// Requires: BOT_TOKEN env var (from @BotFather)
//           INDEXER_URL env var (default: http://localhost:8443)
//
// Run: BOT_TOKEN=xxx node src/index.js

import { Bot, InlineKeyboard } from "grammy";
import { ApiClient } from "./lib/api.js";
import { encrypt, decrypt, getEncryptionKey } from "./crypto.js";
import { readFile, writeFile } from "fs/promises";
import { existsSync } from "fs";

const token = process.env.BOT_TOKEN;
if (!token) {
	console.error("BOT_TOKEN environment variable required");
	process.exit(1);
}

const apiUrl = process.env.INDEXER_URL || "http://localhost:8443";
const api = new ApiClient(apiUrl);
const bot = new Bot(token);

// ── Encryption at rest ──────────────────────────────────────────────
const KEY_BYTES = getEncryptionKey();
if (!KEY_BYTES) {
	console.warn(
		"⚠️  BOT_ENCRYPTION_KEY not set — user data stored in plaintext. Set it with: openssl rand -base64 32",
	);
}

// ── User address storage (encrypted at rest) ────────────────────────
const USERS_FILE = "/tmp/daglock-users.json";
let users = {};

async function loadUsers() {
	try {
		if (existsSync(USERS_FILE)) {
			const data = await readFile(USERS_FILE, "utf-8");
			// Try parsing — if it fails, data might be corrupted from encryption migration
			const raw = JSON.parse(data);
			// Decrypt each entry if needed
			for (const [id, entry] of Object.entries(raw)) {
				if (entry.encryptedAddress) {
					users[id] = {
						address: decrypt(entry.encryptedAddress, KEY_BYTES),
						updatedAt: entry.updatedAt,
					};
				} else if (entry.address) {
					// Legacy plaintext entry — keep as-is until next save
					users[id] = entry;
				}
			}
		}
	} catch (e) {
		users = {};
	}
}

async function saveUsers() {
	try {
		// Encrypt each entry before saving
		const encrypted = {};
		for (const [id, entry] of Object.entries(users)) {
			encrypted[id] = {
				encryptedAddress: encrypt(entry.address, KEY_BYTES),
				updatedAt: entry.updatedAt,
			};
		}
		await writeFile(USERS_FILE, JSON.stringify(encrypted, null, 2));
	} catch (e) {
		console.error("Failed to save users:", e.message);
	}
}

function getUserAddress(userId) {
	return users[userId]?.address;
}

// In-memory conversation wizard state
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
function advanceConv(userId, step) {
	const s = convState.get(userId);
	if (s) s.step = step;
}
function endConv(userId) {
	convState.delete(userId);
}

function setUserAddress(userId, address) {
	users[userId] = { address, updatedAt: Date.now() };
	saveUsers();
}

await loadUsers();

// ── Commands ─────────────────────────────────────────────────────────

bot.api.setMyCommands([
	{ command: "start", description: "Start the bot or claim an escrow" },
	{ command: "setaddress", description: "Set your Kaspa address" },
	{ command: "create", description: "Create a new escrow" },
	{ command: "claim", description: "Claim an escrow by ID" },
	{ command: "settle", description: "Settle an active escrow" },
	{ command: "refund", description: "Refund an escrow" },
	{ command: "submit_tx", description: "Submit TX ID after broadcasting" },
	{ command: "submit_sig", description: "Submit signature for settle/refund" },
	{ command: "list", description: "List your escrows" },
	{ command: "offers", description: "Browse open offers" },
	{ command: "counter", description: "Counter an offer with a different amount" },
	{ command: "counters", description: "List counter-offers on an offer" },
	{ command: "swap", description: "Settle an escrow via atomic swap" },
	{ command: "vaults", description: "List your vaults" },
	{ command: "reputation", description: "Check counterparty reputation" },
	{ command: "status", description: "Check escrow status" },
	{ command: "receipt", description: "Fetch a settlement receipt" },
	{ command: "dispute", description: "Dispute an escrow" },
	{ command: "cancel", description: "Cancel an escrow" },
	{ command: "msg", description: "Send a message on an escrow" },
	{ command: "messages", description: "Read message thread on an escrow" },
	{ command: "evidence", description: "List evidence for an escrow" },
	{ command: "mediate", description: "Start AI mediation on a disputed escrow" },
	{ command: "mediate_accept", description: "Accept AI mediation outcome" },
	{ command: "mediate_status", description: "Check AI mediation status" },
	{ command: "fee", description: "Calculate escrow fee for an amount" },
	{ command: "block", description: "Block a user (via web)" },
	{ command: "feedback", description: "Leave trade feedback (via web)" },
	{ command: "create_milestone", description: "Create a milestone escrow" },
	{ command: "milestones", description: "List your milestone escrows" },
	{ command: "release_milestone", description: "Release current milestone" },
	{ command: "my_escalations", description: "Check your dispute escalation status" },
	{ command: "create_multi", description: "Create a multi-party escrow" },
	{ command: "multi_escrows", description: "List your multi-party escrows" },
	{ command: "sign", description: "Sign a multi-party escrow release" },
	{ command: "help", description: "Show help" },
]);

bot.command("start", async (ctx) => {
	const payload = ctx.match;
	if (payload?.startsWith("claim_")) {
		const escrowId = payload.replace("claim_", "");
		return handleClaim(ctx, escrowId);
	}

	if (payload?.startsWith("swap_")) {
		const escrowId = payload.replace("swap_", "");
		const webUrl = `https://daglock.com/swap/${escrowId}`;
		return await ctx.reply(
			`🔄 *Atomic Swap*\n\n` +
				`Escrow ID: \`${escrowId}\`\n\n` +
				`To claim this swap, open the link below and enter your preimage:\n` +
				`🔗 ${webUrl}\n\n` +
				`_The covenant verifies SHA-256(preimage) matches the trade hash._`,
			{ parse_mode: "Markdown" },
		);
	}

	const address = getUserAddress(ctx.from.id);
	const addressStatus = address
		? `\n📍 Your address: \`${address.slice(0, 20)}...\``
		: "\n⚠️ No address set. Use /setaddress to set your Kaspa address.";

	await ctx.reply(
		"🔒 *DagLock — Trustless Escrow on Kaspa*\n\n" +
			"I help you create and manage trustless escrows using Kaspa smart contracts.\n\n" +
			"_No one can steal your funds — not even me._\n" +
			addressStatus +
			"\n\n" +
			"*Commands:*\n" +
			"/setaddress — Set your Kaspa address\n" +
			"/create — Create an escrow\n" +
			"/claim <id> — Claim an escrow\n" +
			"/list — List your escrows\n" +
			"/offers — Browse offers\n" +
			"/swap <id> <hex> — Atomic swap settle\n" +
			"/vaults — List your vaults\n" +
			"/status <id> — Check escrow\n" +
			"/receipt <id> — Fetch receipt\n" +
			"/dispute <id> <reason> — Dispute escrow\n" +
			"/mediate <id> <claim> — Start AI mediation\n" +
			"/mediate_accept <id> — Accept mediation outcome\n" +
			"/mediate_status <id> — Check mediation status\n" +
			"/cancel <id> — Cancel escrow\n" +
			"/reputation <address> — Check reputation\n" +
			"/fee <amount> — Calculate escrow fee\n" +
			"/block <address> — Block a user\n" +
			"/feedback <id> <rating> — Leave trade feedback\n" +
			"/msg <id> <text> — Send message on escrow\n" +
			"/messages <id> — Read message thread",
		{ parse_mode: "Markdown" },
	);
});

bot.command("setaddress", async (ctx) => {
	const address = ctx.match?.trim();
	if (!address) {
		return await ctx.reply(
			"📍 *Set Your Address*\n\n" +
				"Usage: /setaddress <kaspa-address>\n\n" +
				"Example:\n" +
				"`/setaddress kaspa:qdyzkrhd74v6cetrv4fhv`",
			{ parse_mode: "Markdown" },
		);
	}

	if (!address.startsWith("kaspa:")) {
		return await ctx.reply("❌ Address must start with `kaspa:`", {
			parse_mode: "Markdown",
		});
	}

	setUserAddress(ctx.from.id, address);
	await ctx.reply(
		`✅ Address set!\n\nYour Kaspa address is now: \`${address}\``,
		{ parse_mode: "Markdown" },
	);
});

bot.command("create", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"Please set your address first:\n/setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}
	startConv(ctx.from.id);
	updateConv(ctx.from.id, "address", address);
	await ctx.reply(
		"🔄 *Create Escrow - Step 1/5*\n\nHow much KAS do you want to escrow?\n\nExample: `100` or `5000.5`\n\n_Type /cancel to cancel at any step._",
		{ parse_mode: "Markdown" },
	);
});

bot.command("invoice", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"Please set your address first:\n/setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}
	startConv(ctx.from.id);
	updateConv(ctx.from.id, "address", address);
	await ctx.reply(
		"*Create Invoice - Step 1/3*\n\nHow much KAS is the invoice for?\n\nExample: `100` or `2500`",
		{ parse_mode: "Markdown" },
	);
});

bot.command("settle", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /settle <escrow-id>");
	return handleSettle(ctx, id);
});

bot.command("refund", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /refund <escrow-id>");
	return handleRefund(ctx, id);
});

bot.command("cancel", async (ctx) => {
	// If in a wizard, cancel it
	const conv = getConv(ctx.from.id);
	if (conv) {
		endConv(ctx.from.id);
		return await ctx.reply("❌ Wizard cancelled.", { parse_mode: "Markdown" });
	}
	// Otherwise cancel an escrow
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /cancel <escrow-id>");
	return handleCancelEscrow(ctx, id);
});

bot.command("claim", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /claim <escrow-id>");
	return handleClaim(ctx, id);
});

bot.command("list", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first:\n`/setaddress <kaspa-address>`",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const data = await api.listEscrows(address);
		const escrows = data.escrows || [];

		if (escrows.length === 0) {
			return await ctx.reply("📭 No escrows found for your address.");
		}

		let msg = "📋 *Your Escrows*\n\n";
		for (const e of escrows.slice(0, 5)) {
			const amount = (e.amount_sompi / 1e8).toFixed(2);
			const statusEmoji = {
				pending_confirmation: "⏳",
				active: "✅",
				settled: "🎉",
				refunded: "↩️",
				disputed: "⚠️",
				cancelled: "🛑",
				expired: "⏰",
			};
			msg += `${statusEmoji[e.status] || "❓"} *${amount} KAS* — \`${e.status}\`\n`;
			msg += `  ID: \`${e.id}\`\n\n`;
		}
		if (escrows.length > 5) msg += `_...and ${escrows.length - 5} more_\n`;
		msg += "💡 Use /status <id> for details";

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch escrows: " + err.message);
	}
});

bot.command("offers", async (ctx) => {
	try {
		const data = await api.listOffers();
		const offers = data.offers || [];

		if (offers.length === 0) {
			return await ctx.reply("📭 No open offers right now.");
		}

		let msg = "📋 *Open Offers*\n\n";
		for (const o of offers.slice(0, 5)) {
			const amount = (o.amount_sompi / 1e8).toFixed(2);
			msg += `• *${o.side.toUpperCase()}* ${amount} ${o.base_asset} for ${o.quote_asset}\n`;
			msg += `  ID: \`${o.id}\`\n`;
			msg += `  Creator: \`${(o.creator_address || "").slice(0, 16)}...\`\n\n`;
		}
		if (offers.length > 5) msg += `_...and ${offers.length - 5} more_\n`;
		msg += "💡 Use /counter <offer-id> <amount> [msg] to counter";

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch offers: " + err.message);
	}
});

// ── Counter-offer commands ─────────────────────────────────────────

bot.command("counter", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"Please set your address first:\n/setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	const [offerId, amountStr, ...msgParts] = (ctx.match || "").trim().split(/\s+/);
	const amount = parseFloat(amountStr);
	const message = msgParts.join(" ").trim();

	if (!offerId || isNaN(amount) || amount <= 0) {
		return await ctx.reply(
			"*Counter an Offer*\n\n" +
				"Usage: `/counter <offer-id> <amount> [message]`\n\n" +
				"Example:\n" +
				"`/counter off_abc123 950 Can you do 950 KAS?`\n\n" +
				"Check /offers for available offers.",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const sompiAmount = Math.round(amount * 100_000_000);
		const result = await api.counterOffer(offerId, sompiAmount, message || undefined, address);
		await ctx.reply(
			`✅ *Counter-offer submitted!*\n\n` +
				`Offer: \`${offerId}\`\n` +
				`Amount: ${amount.toFixed(2)} KAS\n` +
				(message ? `Message: ${message}\n` : "") +
				`\nOffer creator can accept via web or /counters ${offerId}`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Failed: " + err.message);
	}
});

bot.command("counters", async (ctx) => {
	const offerId = ctx.match?.trim();
	if (!offerId) return await ctx.reply("Usage: /counters <offer-id>");

	try {
		const data = await api.listCounters(offerId);
		const counters = data.counters || [];

		if (counters.length === 0) {
			return await ctx.reply(`📭 No counter-offers for \`${offerId}\`.`, {
				parse_mode: "Markdown",
			});
		}

		let msg = `📋 *Counter-offers for \`${offerId}\`*\n\n`;
		for (const c of counters.slice(0, 5)) {
			const amount = c.amount_sompi ? (c.amount_sompi / 1e8).toFixed(2) : "—";
			const proposer = (c.proposer_address || "").slice(0, 16);
			msg += `• *${amount} KAS* by \`${proposer}...\`\n`;
			msg += `  Status: \`${c.status}\`\n`;
			if (c.message) msg += `  "${c.message}"\n`;
			msg += `  ID: \`${c.id}\`\n\n`;
		}
		if (counters.length > 5) msg += `_...${counters.length - 5} more_\n`;
		msg += "💡 Accept via web or use /accept_counter <counter-id>";

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch counters: " + err.message);
	}
});

bot.command("status", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /status <escrow-id>");

	try {
		const data = await api.getEscrow(id);
		const amount = (data.amount_sompi / 1e8).toFixed(2);
		const fee = (data.fee_sompi / 1e8).toFixed(4);
		const created = new Date(data.created_at * 1000)
			.toISOString()
			.slice(0, 19)
			.replace("T", " ");

		const statusEmoji = {
			pending_confirmation: "⏳",
			active: "✅",
			settled: "🎉",
			refunded: "↩️",
			disputed: "⚠️",
			cancelled: "🛑",
			expired: "⏰",
		};

		await ctx.reply(
			`*Escrow: ${id}*\n\n` +
				`Status: ${statusEmoji[data.status] || "❓"} \`${data.status}\`\n` +
				`Amount: ${amount} KAS\n` +
				`Fee: ${fee} KAS (0.5%)\n` +
				`Buyer: \`${data.buyer_address?.slice(0, 16) || "N/A"}...\`\n` +
				`Created: ${created} UTC` +
				(data.seller_address
					? `\nSeller: \`${data.seller_address.slice(0, 16)}...\``
					: "") +
				(data.dispute_reason ? `\nReason: ${data.dispute_reason}` : ""),
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Escrow not found: " + err.message);
	}
});

bot.command("receipt", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /receipt <escrow-id>");

	try {
		const receipt = await api.getReceipt(id);
		await ctx.reply(
			`🧾 *Receipt*\n\n` +
				`Receipt ID: \`${receipt.receipt_id}\`\n` +
				`Escrow: \`${receipt.escrow_id}\`\n` +
				`Status: \`${receipt.status}\`\n` +
				`Asset: ${receipt.asset}\n` +
				`Amount: ${receipt.amount_sompi} units\n` +
				`Fee: ${receipt.fee_sompi} units` +
				(receipt.dispute_reason ? `\nReason: ${receipt.dispute_reason}` : ""),
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Receipt not found: " + err.message);
	}
});

bot.command("dispute", async (ctx) => {
	const [id, ...reasonParts] = (ctx.match || "").trim().split(/\s+/);
	const reason = reasonParts.join(" ").trim();
	if (!id || !reason)
		return await ctx.reply("Usage: /dispute <escrow-id> <reason>");

	try {
		const result = await api.disputeEscrow(id, reason);
		const caseId = result.jury_case_id ? `\nCase: \`${result.jury_case_id}\`` : "";
		await ctx.reply(
			`⚠️ *Escrow Disputed*\n\n` +
				`ID: \`${result.escrow_id}\`\n` +
				`Reason: ${reason}\n\n` +
				`*Escalation Tiers:*\n` +
				`Mediation (2 days) → Jury Vote (5 days) → Admin Override (10 days)\n` +
				`Use /my_escalations to check status.${caseId}`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not dispute: " + err.message);
	}
});

bot.command("cancel", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /cancel <escrow-id>");

	try {
		const result = await api.cancelEscrow(id);
		await ctx.reply(
			`🛑 *Escrow Cancelled*\n\n` + `ID: \`${result.escrow_id}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not cancel: " + err.message);
	}
});

bot.command("reputation", async (ctx) => {
	const address = ctx.match?.trim();
	if (!address) return await ctx.reply("Usage: /reputation <kaspa-address>");

	try {
		const rep = await api.getReputation(address);
		const volume = (rep.total_volume_sompi / 1e8).toFixed(2);
		const shield = "🛡️".repeat(Math.min(5, Math.ceil(rep.score)));

		await ctx.reply(
			`📊 *Reputation*\n\n` +
				`Address: \`${address.slice(0, 20)}...\`\n` +
				`Score: ${shield} ${rep.score.toFixed(2)}/5\n\n` +
				`*Stats:*\n` +
				`Trades: ${rep.trade_count}\n` +
				`Settled: ${rep.settled_count}\n` +
				`Refunded: ${rep.refunded_count}\n` +
				`Disputed: ${rep.disputed_count}\n` +
				`Dispute Rate: ${(rep.dispute_rate * 100).toFixed(1)}%\n` +
				`Volume: ${volume} KAS`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
});

// ── Swap command ────────────────────────────────────────────────────

bot.command("swap", async (ctx) => {
	const [id, ...preimageParts] = (ctx.match || "").trim().split(/\s+/);
	const preimage = preimageParts.join(" ").trim();
	if (!id || !preimage) {
		return await ctx.reply("Usage: /swap <escrow-id> <preimage-hex>");
	}

	try {
		const result = await api.swapEscrow(id, preimage);
		await ctx.reply(
			`✅ *Atomic Swap Settled*\n\n` +
				`Escrow: \`${result.escrow_id}\`\n` +
				`Method: \`${result.method || "atomic_swap"}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Swap failed: " + err.message);
	}
});

// ── Fee calculator ──────────────────────────────────────────────────

bot.command("fee", async (ctx) => {
	const amountStr = ctx.match?.trim();
	if (!amountStr || isNaN(Number(amountStr)) || Number(amountStr) <= 0) {
		return await ctx.reply(
			"*Fee Calculator*\n\n" +
				"Usage: `/fee <amount-in-kas>`\n\n" +
				"Example: `/fee 1000`\n\n" +
				"DagLock charges a **0.5% protocol fee** (1/200) on settlement.",
			{ parse_mode: "Markdown" },
		);
	}

	const amount = Number(amountStr);
	const fee = amount / 200;
	const net = amount - fee;

	// Try to fetch USD price for reference
	let usdLine = "";
	try {
		const price = await api.request("/network/price");
		if (price?.kas_usd) {
			usdLine = `\nUSD value: ~$${(amount * price.kas_usd).toLocaleString()}`;
		}
	} catch {
		// price fetch is optional
	}

	await ctx.reply(
		`🧮 *Fee Estimate for ${amount.toLocaleString()} KAS*\n\n` +
			`Protocol fee (0.5%): **${fee.toLocaleString()} KAS**\n` +
			`Net to seller: **${net.toLocaleString()} KAS**\n` +
			`Treasury receives: **${fee.toLocaleString()} KAS**` +
			usdLine +
			`\n\n_Fee is enforced by the covenant — cannot be bypassed._`,
		{ parse_mode: "Markdown" },
	);
});

// ── Block user ──────────────────────────────────────────────────────

bot.command("block", async (ctx) => {
	const address = ctx.match?.trim();
	if (!address) {
		return await ctx.reply(
			"*Block User*\n\n" +
				"Usage: /block <kaspa-address>\n\n" +
				"Blocking a user hides their escrows and offers from your view.\n\n" +
				"To block, use the web interface: https://daglock.com/reputation\n" +
				"(Blocking requires message signing — not available in Telegram)",
			{ parse_mode: "Markdown" },
		);
	}

	await ctx.reply(
		`To block \`${address.slice(0, 20)}...\`, please use the web interface:\n\n` +
			`🔗 https://daglock.com/reputation?block=${encodeURIComponent(address)}\n\n` +
			"_Blocking requires signing a message with your wallet._",
		{ parse_mode: "Markdown" },
	);
});

// ── Trade feedback ──────────────────────────────────────────────────

bot.command("feedback", async (ctx) => {
	const parts = (ctx.match || "").trim().split(/\s+/);
	const escrowId = parts[0];
	const rating = parseInt(parts[1], 10);
	const comment = parts.slice(2).join(" ");

	if (!escrowId || isNaN(rating) || rating < 1 || rating > 5) {
		return await ctx.reply(
			"*Trade Feedback*\n\n" +
				"Usage: `/feedback <escrow-id> <rating 1-5> [comment]`\n\n" +
				"Leave feedback after an escrow settles.\n\n" +
				"_Feedback requires message signing — use the web interface._\n" +
				`🔗 https://daglock.com/escrows`,
			{ parse_mode: "Markdown" },
		);
	}

	// Link to web for signing
	await ctx.reply(
		`To leave feedback for escrow \`${escrowId}\`:\n\n` +
			`🔗 https://daglock.com/escrows?id=${encodeURIComponent(escrowId)}\n\n` +
			"_Feedback requires message signing with your wallet._",
		{ parse_mode: "Markdown" },
	);
});

// ── Vault commands ──────────────────────────────────────────────────

bot.command("vaults", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
		);
	}

	try {
		const data = await api.listVaults(address);
		const vaults = data.vaults || [];

		if (vaults.length === 0) {
			return await ctx.reply("📭 No vaults found for your address.");
		}

		let msg = "🏦 *Your Vaults*\n\n";
		for (const v of vaults.slice(0, 5)) {
			const amount = (v.amount_sompi / 1e8).toFixed(2);
			const statusEmoji = {
				locked: "🔒",
				unlocked: "🔓",
				expired: "⏰",
				transferred: "↗️",
			};
			msg += `${statusEmoji[v.status] || "❓"} *${amount} KAS* — \`${v.status}\`\n`;
			msg += `  ID: \`${v.id}\``;
			if (v.timeout) {
				const remaining = Math.max(
					0,
					v.timeout - Math.floor(Date.now() / 1000),
				);
				if (remaining > 0)
					msg += `\n  Unlocks: ~${Math.ceil(remaining / 86400)}d`;
			}
			msg += "\n\n";
		}
		if (vaults.length > 5) msg += `_...and ${vaults.length - 5} more_\n`;

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch vaults: " + err.message);
	}
});

// ── Message commands ────────────────────────────────────────────────

bot.command("msg", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
		);
	}

	const [id, ...textParts] = (ctx.match || "").trim().split(/\s+/);
	const text = textParts.join(" ").trim();
	if (!id || !text)
		return await ctx.reply("Usage: /msg <escrow-id> <message-text>");

	try {
		const result = await api.sendMessage(id, text);
		await ctx.reply(
			`💬 Message sent on \`${result.message?.escrow_id || id}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not send message: " + err.message);
	}
});

// TODO: Messages may now be E2E encrypted (content_enc + nonce fields).
// The bot cannot decrypt them. Consider showing:
//   "View encrypted messages on the web dashboard: https://daglock.com/escrows/{id}"
// when content_enc is present instead of raw content.
bot.command("messages", async (ctx) => {
	const [id] = (ctx.match || "").trim().split(/\s+/);
	if (!id) return await ctx.reply("Usage: /messages <escrow-id>");

	try {
		const data = await api.listMessages(id);
		const msgs = data.messages || [];

		if (msgs.length === 0) {
			return await ctx.reply("💬 No messages on this escrow.");
		}

		let msg = `💬 *Messages on \`${id}\`*\n\n`;
		for (const m of msgs.slice(-5)) {
			const date = new Date(m.created_at * 1000).toISOString().slice(11, 19);
			const sender = (m.sender_address || "").slice(0, 12);
			if (m.content_enc) {
				msg += `[${date}] \`${sender}...\`: 🔒 Encrypted — view on web\n`;
			} else {
				msg += `[${date}] \`${sender}...\`: ${m.content}\n`;
			}
		}
		if (msgs.length > 5)
			msg += `\n_...${msgs.length - 5} older messages hidden_`;

		msg += `\n\n🔒 E2E encrypted messages are only viewable on the web dashboard:\nhttps://daglock.com/escrows/${id}`;

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch messages: " + err.message);
	}
});

// ── Evidence command ────────────────────────────────────────────────

bot.command("evidence", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /evidence <escrow-id>");

	try {
		const data = await api.listEvidence(id);
		const evidence = data.evidence || [];

		if (evidence.length === 0) {
			return await ctx.reply("📄 No evidence submitted for this escrow.");
		}

		let msg = `📄 *Evidence for \`${id}\`*\n\n`;
		for (const ev of evidence.slice(-5)) {
			const date = new Date(ev.created_at * 1000)
				.toISOString()
				.slice(0, 19)
				.replace("T", " ");
			const by = (ev.submitted_by || "").slice(0, 16);
			const content = (ev.content || "").slice(0, 200);
			msg += `[${date}] \`${by}...\`: ${content}\n\n`;
		}
		if (evidence.length > 5) msg += `_...${evidence.length - 5} older items_\n`;

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch evidence: " + err.message);
	}
});

// ── AI Mediation commands ────────────────────────────────────────────

bot.command("mediate", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
		);
	}

	const [id, ...claimParts] = (ctx.match || "").trim().split(/\s+/);
	const claim = claimParts.join(" ").trim();
	if (!id || !claim) {
		return await ctx.reply(
			"*AI Mediation*\n\n" +
			"Usage: `/mediate <escrow-id> <your-claim>`\n\n" +
			"Start non-binding AI mediation on a disputed escrow. The AI analyzes the dispute and proposes a fair outcome.\n\n" +
			"Example:\n" +
			"`/mediate esc_abc123 I paid 500 KAS but never received the item`",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const auth = api.makeAuth(address, "mediation:submit");
		const body = { buyer_claim: claim, seller_claim: "" };
		const result = await api.mediateEscrow(id, body.buyer_claim, body.seller_claim, auth);
		await ctx.reply(
			`🤖 *AI Mediation Started*\n\n` +
			`Escrow: \`${id}\`\n` +
			`Status: \`${result.mediation_status}\`\n\n` +
			`The AI is analyzing the dispute. Use /mediate_status ${id} to check the recommendation.\n\n` +
			`_Mediation is non-binding. If not accepted within 24h, it escalates to jury._`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Failed: " + err.message);
	}
});

bot.command("mediate_accept", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
		);
	}

	const [id] = (ctx.match || "").trim().split(/\s+/);
	if (!id) {
		return await ctx.reply(
			"*Accept Mediation*\n\n" +
			"Usage: `/mediate_accept <escrow-id>`\n\n" +
			"Accept the AI mediation recommendation for a disputed escrow.\n\n" +
			"Example:\n" +
			"`/mediate_accept esc_abc123`",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const auth = api.makeAuth(address, `mediation:accept:${id}`);
		const result = await api.acceptMediation(id, "buyer", auth);
		if (result.outcome_executed) {
			await ctx.reply(
				`✅ *Mediation Outcome Executed!*\n\n` +
				`Both parties accepted. The outcome has been applied to escrow \`${id}\`.`,
				{ parse_mode: "Markdown" },
			);
		} else {
			await ctx.reply(
				`✅ *Mediation Accepted*\n\n` +
				`Escrow: \`${id}\`\n` +
				`Waiting for the other party to accept.\n\n` +
				`_If not accepted within 24h, it escalates to jury._`,
				{ parse_mode: "Markdown" },
			);
		}
	} catch (err) {
		await ctx.reply("❌ Failed: " + err.message);
	}
});

bot.command("mediate_status", async (ctx) => {
	const [id] = (ctx.match || "").trim().split(/\s+/);
	if (!id) {
		return await ctx.reply("Usage: /mediate_status <escrow-id>");
	}

	try {
		const data = await api.getMediation(id);
		const status = data.mediation_status || "unknown";
		const outcome = data.recommendation
			? `\n\n*Proposed Outcome:* ${data.recommendation.outcome.toUpperCase()}` +
			  (data.recommendation.outcome === "split"
			  	? ` (${(data.recommendation.buyer_share_basis / 100).toFixed(1)}% buyer)`
			  	: "") +
			  `\n_${data.recommendation.reasoning.slice(0, 300)}_`
			: "";
		const acceptStatus = `\n\nBuyer accepted: ${data.buyer_accepted ? "✅" : "❌"} · Seller accepted: ${data.seller_accepted ? "✅" : "❌"}`;
		const remaining = data.expires_at
			? `\n⏰ Expires: ${Math.ceil(Math.max(0, data.expires_at * 1000 - Date.now()) / 3600000)}h`
			: "";

		await ctx.reply(
			`🤖 *AI Mediation Status*\n\n` +
			`Escrow: \`${id}\`\n` +
			`Status: \`${status}\`` +
			outcome +
			acceptStatus +
			remaining +
			`\n\n/mediate_accept ${id} — Accept the outcome`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not fetch mediation: " + err.message);
	}
});

// ── Escalation check ────────────────────────────────────────────────

bot.command("my_escalations", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
		);
	}

	try {
		const data = await api.request(`/jury/cases/active/${encodeURIComponent(address)}`);
		const cases = data.cases || [];
		const escalationLabels = ["Mediation (2d)", "Jury Vote (5d)", "Admin Override (10d)"];

		if (cases.length === 0) {
			return await ctx.reply("✅ No active jury cases with escalations.");
		}

		let msg = "⚖ *Dispute Escalations*\n\n";
		for (const c of cases) {
			const label = escalationLabels[c.escalation_level] || "Unknown";
			const deadline = c.escalation_deadline
				? new Date(c.escalation_deadline * 1000).toISOString().slice(0, 19).replace("T", " ")
				: "N/A";
			msg += `*${c.id.slice(0, 16)}…* — ${label}\n`;
			msg += `Status: \`${c.status}\` · Deadline: ${deadline}\n`;
			msg += `Votes: ${c.votes_for_seller + c.votes_for_buyer}/${c.juror_count}\n\n`;
		}
		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch escalations: " + err.message);
	}
});

// ── Multi-party escrow commands ────────────────────────────────────

bot.command("create_multi", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"Please set your address first:\n/setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	const parts = (ctx.match || "").trim().split(/\s+/);
	const lockTxId = parts[0];
	const totalAmountStr = parts[1];

	if (!lockTxId || !totalAmountStr) {
		return await ctx.reply(
			"*Create Multi-Party Escrow*\n\n" +
			"Usage: `/create_multi <lock-tx-id> <total-kas> <party1-address> <share1-pct> <party2-address> <share2-pct> [...]`\n\n" +
			"Shares must sum to 100%. Supports 2-4 parties.\n\n" +
			"Example:\n" +
			"`/create_multi tx123 10000 kaspa:p1... 50 kaspa:p2... 30 kaspa:p3... 20`",
			{ parse_mode: "Markdown" },
		);
	}

	const totalAmount = parseFloat(totalAmountStr);
	if (isNaN(totalAmount) || totalAmount <= 0) {
		return await ctx.reply("Invalid total amount.", { parse_mode: "Markdown" });
	}

	const partyArgs = parts.slice(2);
	if (partyArgs.length < 4 || partyArgs.length % 2 !== 0 || partyArgs.length / 2 > 4) {
		return await ctx.reply("Provide pairs of <address> <share-pct> for 2-4 parties.", { parse_mode: "Markdown" });
	}

	const parties = [];
	const sharePcts = [];
	for (let i = 0; i < partyArgs.length; i += 2) {
		const addr = partyArgs[i];
		const pct = parseFloat(partyArgs[i + 1]);
		if (!addr.startsWith("kaspa:") || isNaN(pct) || pct <= 0) {
			return await ctx.reply(`Invalid party: "${addr}" or share "${partyArgs[i + 1]}"`, { parse_mode: "Markdown" });
		}
		parties.push(addr);
		sharePcts.push(pct);
	}

	const totalPct = sharePcts.reduce((a, b) => a + b, 0);
	if (Math.abs(totalPct - 100) > 0.01) {
		return await ctx.reply(`Shares must sum to 100%, got ${totalPct.toFixed(2)}%`, { parse_mode: "Markdown" });
	}

	const shares = sharePcts.map((p) => Math.round(p * 100));
	const sompiTotal = Math.round(totalAmount * 100_000_000);

	try {
		const result = await api.createMultiEscrow({
			lock_tx_id: lockTxId,
			parties,
			shares,
			total_amount: sompiTotal,
		});
		await ctx.reply(
			`✅ *Multi-Party Escrow Created!*\n\n` +
			`ID: \`${result.id}\`\n` +
			`Amount: ${totalAmount} KAS\n` +
			`Parties: ${result.parties.length}\n` +
			`Status: \`${result.status}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
});

bot.command("multi_escrows", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const data = await api.listMultiEscrows(address);
		const escrows = data.multi_escrows || [];

		if (escrows.length === 0) {
			return await ctx.reply("📭 No multi-party escrows found.");
		}

		let msg = "👥 *Your Multi-Party Escrows*\n\n";
		for (const m of escrows.slice(0, 5)) {
			const amount = (m.total_amount / 1e8).toFixed(2);
			const signed = m.signatures.length;
			const total = m.parties.length;
			msg += `*${amount} KAS* — \`${m.status}\`\n`;
			msg += `  Signatures: ${signed}/${total} · ID: \`${m.id}\`\n\n`;
		}
		if (escrows.length > 5) msg += `_...and ${escrows.length - 5} more_\n`;

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch multi-party escrows: " + err.message);
	}
});

bot.command("sign", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	const [id] = (ctx.match || "").trim().split(/\s+/);
	if (!id) {
		return await ctx.reply(
			"*Sign Multi-Party Escrow*\n\n" +
			"Usage: `/sign <multi-escrow-id>`\n\n" +
			"Signs the release of a multi-party escrow. Funds are distributed when all parties sign.\n\n" +
			"Example:\n" +
			"`/sign multi_abc123`",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const result = await api.signMultiEscrow(id, address);
		const allSigned = result.all_signed ? " ✅ All parties have signed! Use /multi_escrows to see status." : "";
		await ctx.reply(
			`✍️ *Signed Multi-Party Escrow*\n\n` +
			`ID: \`${id}\`\n` +
			`Signatures: ${result.signature_count}/${result.parties_count}` +
			allSigned,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not sign: " + err.message);
	}
});

// ── Milestone commands ──────────────────────────────────────────────

bot.command("create_milestone", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"Please set your address first:\n/setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	const parts = (ctx.match || "").trim().split(/\s+/);
	const sellerAddress = parts[0];
	const totalAmountStr = parts[1];
	const countStr = parts[2];

	if (!sellerAddress || !totalAmountStr || !countStr) {
		return await ctx.reply(
			"*Create Milestone Escrow*\n\n" +
				"Usage: `/create_milestone <seller-address> <total-kas> <milestone-count>`\n\n" +
				"Then follow the prompts for each milestone amount and timeout.\n\n" +
				"Example:\n" +
				"`/create_milestone kaspa:q... 3000 3`",
			{ parse_mode: "Markdown" },
		);
	}

	const totalAmount = parseFloat(totalAmountStr);
	const count = parseInt(countStr, 10);
	if (isNaN(totalAmount) || totalAmount <= 0) {
		return await ctx.reply("Invalid total amount.", { parse_mode: "Markdown" });
	}
	if (isNaN(count) || count < 1 || count > 5) {
		return await ctx.reply("Milestone count must be 1-5.", { parse_mode: "Markdown" });
	}

	startConv(ctx.from.id);
	updateConv(ctx.from.id, "address", address);
	updateConv(ctx.from.id, "sellerAddress", sellerAddress);
	updateConv(ctx.from.id, "totalAmount", totalAmount);
	updateConv(ctx.from.id, "totalSompi", Math.round(totalAmount * 100_000_000));
	updateConv(ctx.from.id, "milestoneCount", count);
	updateConv(ctx.from.id, "milestoneIndex", 0);
	updateConv(ctx.from.id, "milestoneAmounts", []);
	updateConv(ctx.from.id, "milestoneTimeouts", []);

	return await ctx.reply(
		`*Milestone Escrow - Step 1/${count * 2}*\n\n` +
			`Enter amount for milestone 1 (KAS):\n` +
			`Total: ${totalAmount} KAS | Milestones: ${count}`,
		{ parse_mode: "Markdown" },
	);
});

bot.command("milestones", async (ctx) => {
	const address = getUserAddress(ctx.from.id);
	if (!address) {
		return await ctx.reply(
			"⚠️ Please set your address first: /setaddress <kaspa-address>",
			{ parse_mode: "Markdown" },
		);
	}

	try {
		const data = await api.listMilestones(address);
		const milestones = data.milestones || [];

		if (milestones.length === 0) {
			return await ctx.reply("📭 No milestone escrows found.");
		}

		let msg = "🏗️ *Your Milestones*\n\n";
		for (const m of milestones.slice(0, 5)) {
			const amount = (m.total_amount / 1e8).toFixed(2);
			const statusEmoji = {
				active: "🟢",
				completed: "✅",
				disputed: "⚠️",
				refunded: "↩️",
			};
			const progress = m.milestone_statuses.filter((s) => s === "released" || s === "approved").length;
			msg += `${statusEmoji[m.status] || "❓"} *${amount} KAS* — \`${m.status}\`\n`;
			msg += `  ${progress}/${m.milestone_statuses.length} milestones\n`;
			msg += `  ID: \`${m.id}\`\n\n`;
		}
		if (milestones.length > 5) msg += `_...and ${milestones.length - 5} more_\n`;

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch milestones: " + err.message);
	}
});

bot.command("release_milestone", async (ctx) => {
	const id = ctx.match?.trim();
	if (!id) return await ctx.reply("Usage: /release_milestone <milestone-id>");

	try {
		const result = await api.releaseMilestone(id);
		await ctx.reply(
			`✅ *Milestone Released*\n\n` +
				`Milestone: \`${result.escrow_id}\`\n` +
				`Status: \`${result.status}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not release milestone: " + err.message);
	}
});

// ── Help command ────────────────────────────────────────────────────

bot.command("help", async (ctx) => {
	await ctx.reply(
		"🔒 *DagLock Bot Help*\n\n" +
			"*Setup:*\n" +
			"/setaddress — Set your Kaspa address\n\n" +
			"*Escrow:*\n" +
			"/create — Create escrow (native, no web needed)\n" +
			"/settle <id> — Settle an active escrow\n" +
			"/refund <id> — Refund an escrow\n" +
			"/claim <id> — Claim an escrow from a trade link\n" +
			"/cancel <id> — Cancel an escrow or wizard\n" +
			"/dispute <id> <reason> — Dispute escrow\n" +
			"/list — List your escrows\n" +
			"/status <id> — Check escrow status\n" +
			"/receipt <id> — Fetch receipt\n" +
			"/submit_tx <id> <txid> — Submit TX after broadcasting\n" +
			"/submit_sig <id> <sig> — Submit signature for settle/refund\n\n" +
			"*Trading:*\n" +
			"/offers — Browse open offers\n" +
			"/counter <id> <amount> — Counter an offer\n" +
			"/counters <id> — List counters on an offer\n" +
			"/fee <amount> — Calculate escrow fee\n" +
			"/swap <id> <hex> — Atomic swap settle via preimage\n" +
			"/reputation <address> — Check reputation\n" +
			"/vaults — List your vaults\n\n" +
			"*Messaging:*\n" +
			"/msg <id> <text> — Send a message on an escrow\n" +
			"/messages <id> — Read message thread\n\n" +
			"*Trade Links:*\n" +
			"Share a link to let someone claim your escrow:\n" +
			"`https://t.me/DagLock_bot?start=claim_<escrow-id>`",
		{ parse_mode: "Markdown" },
	);
});

// --- Wizard text handler ---
bot.on("message:text", async (ctx, next) => {
	const conv = getConv(ctx.from.id);
	if (!conv) return next();

	const text = ctx.message.text.trim();
	const step = conv.step;

	try {
		// ── Invoice wizard ─────────────────────────────────────────
		if (step === "inv_amount") {
			const num = parseFloat(text);
			if (isNaN(num) || num <= 0) {
				return await ctx.reply("Enter a valid amount, e.g. `100`", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "amount", num);
			advanceConv(ctx.from.id, "inv_description");
			return await ctx.reply(
				"*Create Invoice - Step 2/3*\n\nWhat's this invoice for?\n\nDescribe the work or product (max 200 chars).",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "inv_description") {
			if (text.length < 3 || text.length > 200) {
				return await ctx.reply("Description must be 3-200 characters.", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "description", text);
			advanceConv(ctx.from.id, "inv_due");
			return await ctx.reply(
				"*Create Invoice - Step 3/3*\n\nDue date? (optional)\n\nEnter days from now (e.g. `7` for 7 days), or type `none` for no due date.",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "inv_due") {
			const data = conv.data;
			endConv(ctx.from.id);

			// Redirect to web app — the bot cannot sign transactions
			const webUrl = process.env.WEB_URL || "https://daglock.com";
			const params = new URLSearchParams({
				amount: data.amount.toString(),
				description: data.description,
			});
			if (text !== "none") {
				const days = parseInt(text);
				if (isNaN(days) || days < 1) {
					return await ctx.reply("Enter a number of days (e.g. `7`) or `none`.", {
						parse_mode: "Markdown",
					});
				}
				params.set("due_days", days.toString());
			}
			const redirectUrl = `${webUrl}/escrows?tab=invoice&${params.toString()}`;

			await ctx.reply(
				"✅ *Invoice params ready!*\n\n" +
					`Amount: ${data.amount} KAS\n` +
					`Description: ${data.description}\n` +
					"\nTo create the invoice, open the web app and sign with your wallet:\n" +
					`[Create Invoice](${redirectUrl})`,
				{ parse_mode: "Markdown", disable_web_page_preview: true },
			);
			return;
		}

		// ── Milestone wizard ──────────────────────────────────────
		if (step && step.startsWith("ms_amount_")) {
			const idx = parseInt(step.replace("ms_amount_", ""), 10);
			const num = parseFloat(text);
			if (isNaN(num) || num <= 0) {
				return await ctx.reply("Enter a valid amount, e.g. `500`", {
					parse_mode: "Markdown",
				});
			}
			const amounts = conv.data.milestoneAmounts || [];
			amounts[idx] = num;
			updateConv(ctx.from.id, "milestoneAmounts", amounts);
			advanceConv(ctx.from.id, `ms_timeout_${idx}`);
			return await ctx.reply(
				`*Milestone Escrow - Milestone ${idx + 1}*\n\nEnter timeout in days for milestone ${idx + 1}:\n\n` +
					`Amount: ${num} KAS\n\n` +
					`Example: \`7\` for 7 days`,
				{ parse_mode: "Markdown" },
			);
		}

		if (step && step.startsWith("ms_timeout_")) {
			const idx = parseInt(step.replace("ms_timeout_", ""), 10);
			const days = parseInt(text, 10);
			if (isNaN(days) || days <= 0) {
				return await ctx.reply("Enter a valid number of days, e.g. `7`", {
					parse_mode: "Markdown",
				});
			}
			const timeouts = conv.data.milestoneTimeouts || [];
			timeouts[idx] = Math.floor(Date.now() / 1000) + days * 86400;
			updateConv(ctx.from.id, "milestoneTimeouts", timeouts);

			const nextIdx = idx + 1;
			const count = conv.data.milestoneCount;

			if (nextIdx < count) {
				advanceConv(ctx.from.id, `ms_amount_${nextIdx}`);
				return await ctx.reply(
					`*Milestone Escrow - Step ${nextIdx * 2 + 1}/${count * 2}*\n\nEnter amount for milestone ${nextIdx + 1} (KAS):`,
					{ parse_mode: "Markdown" },
				);
			}

			// All milestones configured — review
			advanceConv(ctx.from.id, "ms_review");
			const amounts = conv.data.milestoneAmounts;
			const total = amounts.reduce((a, b) => a + b, 0);
			let summary = "";
			for (let i = 0; i < count; i++) {
				const d = Math.round((timeouts[i] - Math.floor(Date.now() / 1000)) / 86400);
				summary += `  M${i + 1}: ${amounts[i]} KAS (${d}d)\n`;
			}
			return await ctx.reply(
				"📋 *Milestone Summary — Review*\n\n" +
					`Total: ${conv.data.totalAmount} KAS\n` +
					`Sum of milestones: ${total.toFixed(2)} KAS\n` +
					`${total !== conv.data.totalAmount ? "⚠️ Sum does not match total!\n" : ""}` +
					`${summary}\n` +
					"Reply `confirm` to create, or `cancel` to cancel.",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "ms_review") {
			if (text.toLowerCase() === "confirm") {
				const amounts = conv.data.milestoneAmounts;
				const timeouts = conv.data.milestoneTimeouts;
				const sompiAmounts = amounts.map((a) => Math.round(a * 100_000_000));

				endConv(ctx.from.id);

				try {
					const lockTxId = `manual_${Date.now()}`;
					const result = await api.createMilestone({
						lock_tx_id: lockTxId,
						buyer_address: conv.data.address,
						seller_address: conv.data.sellerAddress,
						total_amount: conv.data.totalSompi,
						milestone_amounts: sompiAmounts,
						milestone_timeouts: timeouts,
					});
					return await ctx.reply(
						`✅ *Milestone Escrow Created!*\n\n` +
							`ID: \`${result.id}\`\n` +
							`Amount: ${conv.data.totalAmount} KAS\n` +
							`Status: \`${result.status}\`\n` +
							`Milestones: ${result.milestone_amounts.length}`,
						{ parse_mode: "Markdown" },
					);
				} catch (err) {
					return await ctx.reply("❌ Error: " + err.message);
				}
			}
			if (text.toLowerCase() === "cancel") {
				endConv(ctx.from.id);
				return await ctx.reply("❌ Milestone creation cancelled.");
			}
			return await ctx.reply("Reply `confirm` or `cancel`.", { parse_mode: "Markdown" });
		}

		// ── Escrow wizard (native) ────────────────────────────────
		if (step === "amount") {
			const num = parseFloat(text);
			if (isNaN(num) || num <= 0) {
				return await ctx.reply("Enter a valid amount, e.g. `100`", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "amount", num);
			advanceConv(ctx.from.id, "counterparty");

			const keyboard = new InlineKeyboard().text("Skip (no counterparty)", "counterparty_skip");
			return await ctx.reply(
				"🔄 *Create Escrow - Step 2/5*\n\nEnter the seller's Kaspa address, or tap Skip:",
				{ parse_mode: "Markdown", reply_markup: keyboard },
			);
		}

		if (step === "counterparty") {
			if (text !== "skip" && !text.startsWith("kaspa:")) {
				return await ctx.reply("Enter a `kaspa:` address or tap Skip", {
					parse_mode: "Markdown",
				});
			}
			if (text !== "skip") updateConv(ctx.from.id, "seller", text);
			advanceConv(ctx.from.id, "timeout");
			const keyboard = new InlineKeyboard()
				.text("1 hour", "timeout_1h")
				.text("24 hours", "timeout_24h")
				.row()
				.text("3 days", "timeout_3d")
				.text("7 days", "timeout_7d");
			return await ctx.reply(
				"🔄 *Create Escrow - Step 3/5*\n\nChoose a timeout:",
				{ parse_mode: "Markdown", reply_markup: keyboard },
			);
		}

		if (step === "timeout_confirm") {
			const days = parseInt(text);
			if (isNaN(days) || days <= 0) {
				return await ctx.reply("Enter a number of days (e.g. `3`)", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "timeoutDays", days);
			advanceConv(ctx.from.id, "dispute");
			const keyboard = new InlineKeyboard()
				.text("Standard", "dispute_standard")
				.text("Mediator", "dispute_mediator")
				.row()
				.text("Jury", "dispute_jury");
			return await ctx.reply(
				"🔄 *Create Escrow - Step 4/5*\n\nDispute mode:\n\n• Standard — timeout refund\n• Mediator — specific mediator resolves\n• Jury — community vote",
				{ parse_mode: "Markdown", reply_markup: keyboard },
			);
		}

		if (step === "dispute_confirm") {
			const mode = text.toLowerCase();
			if (!["standard", "mediator", "jury"].includes(mode)) {
				return await ctx.reply("Enter `standard`, `mediator`, or `jury`", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "disputeMode", mode);
			advanceConv(ctx.from.id, "review");

			// Show review summary
			const data = conv.data;
			const amount = data.amount;
			const timeoutDays = data.timeoutDays;
			const seller = data.seller || "None (open escrow)";
			const fee = (amount / 200).toFixed(2);
			const net = (amount - parseFloat(fee)).toFixed(2);

			const keyboard = new InlineKeyboard()
				.text("✅ Confirm & Sign", "create_confirm")
				.row()
				.text("❌ Cancel", "create_cancel");

			return await ctx.reply(
				"📋 *Escrow Summary — Review*\n\n" +
					`Amount: **${amount} KAS**\n` +
					`Fee (0.5%): **${fee} KAS**\n` +
					`Net to seller: **${net} KAS**\n` +
					`Seller: \`${seller}\`\n` +
					`Timeout: ${timeoutDays} day(s)\n` +
					`Dispute mode: ${mode}\n\n` +
					"_Tap Confirm to generate the covenant address and sign with your wallet._",
				{ parse_mode: "Markdown", reply_markup: keyboard },
			);
		}
	} catch (e) {
		await ctx.reply("Error: " + e.message);
		endConv(ctx.from.id);
	}
});

// ── Inline keyboard callback handler ────────────────────────────────

bot.on("callback_query:data", async (ctx) => {
	const data = ctx.callbackQuery.data;
	const userId = ctx.from.id;
	const conv = getConv(userId);

	// Handle create wizard callbacks
	if (data === "counterparty_skip") {
		if (!conv) return ctx.answerCallbackQuery("Session expired. Use /create again.");
		advanceConv(userId, "timeout");
		const keyboard = new InlineKeyboard()
			.text("1 hour", "timeout_1h")
			.text("24 hours", "timeout_24h")
			.row()
			.text("3 days", "timeout_3d")
			.text("7 days", "timeout_7d");
		await ctx.editMessageText(
			"🔄 *Create Escrow - Step 3/5*\n\nChoose a timeout:",
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
		return ctx.answerCallbackQuery();
	}

	if (data === "timeout_1h") { updateConv(userId, "timeoutDays", 0.042); advanceConv(userId, "dispute"); return showDisputeStep(ctx); }
	if (data === "timeout_24h") { updateConv(userId, "timeoutDays", 1); advanceConv(userId, "dispute"); return showDisputeStep(ctx); }
	if (data === "timeout_3d") { updateConv(userId, "timeoutDays", 3); advanceConv(userId, "dispute"); return showDisputeStep(ctx); }
	if (data === "timeout_7d") { updateConv(userId, "timeoutDays", 7); advanceConv(userId, "dispute"); return showDisputeStep(ctx); }

	if (data === "dispute_standard") { updateConv(userId, "disputeMode", "standard"); advanceConv(userId, "review"); return showReviewStep(ctx); }
	if (data === "dispute_mediator") { updateConv(userId, "disputeMode", "mediator"); advanceConv(userId, "review"); return showReviewStep(ctx); }
	if (data === "dispute_jury") { updateConv(userId, "disputeMode", "jury"); advanceConv(userId, "review"); return showReviewStep(ctx); }

	if (data === "create_cancel") {
		endConv(userId);
		await ctx.editMessageText("❌ Escrow creation cancelled.", { parse_mode: "Markdown" });
		return ctx.answerCallbackQuery();
	}

	if (data === "create_confirm") {
		if (!conv) {
			await ctx.answerCallbackQuery("Session expired. Use /create again.");
			return;
		}
		await ctx.answerCallbackQuery();
		return handleCreateConfirm(ctx, conv.data);
	}

	// Handle action confirmations
	if (data.startsWith("settle_confirm_")) {
		const escrowId = data.replace("settle_confirm_", "");
		return handleSettleConfirm(ctx, escrowId);
	}
	if (data.startsWith("refund_confirm_")) {
		const escrowId = data.replace("refund_confirm_", "");
		return handleRefundConfirm(ctx, escrowId);
	}

	// Handle sign step callbacks
	if (data === "sign_done") {
		await ctx.answerCallbackQuery();
		const conv = getConv(ctx.from.id);
		if (!conv || conv.step !== "sign") {
			return await ctx.reply("No pending escrow creation. Use /create to start.");
		}
		return await ctx.reply(
			"📤 *Enter TX ID*\n\n" +
				"Paste the transaction ID (txid) from your wallet after broadcasting:\n\n" +
				"`/submit_tx <escrow-id> <txid>`\n\n" +
				"Example:\n" +
				`\`/submit_tx ${conv.data.compileResult?.id || "esc_..."} <paste-txid-here>\``,
			{ parse_mode: "Markdown" },
		);
	}
	if (data === "sign_copy") {
		await ctx.answerCallbackQuery();
		const conv = getConv(ctx.from.id);
		if (!conv || !conv.data.covenantAddress) return;
		return await ctx.reply(
			"📋 *Transaction Data*\n\n" +
				`Send to: \`${conv.data.covenantAddress}\`\n` +
				`Amount: ${conv.data.amount} KAS\n\n` +
				"_Use any Kaspa wallet to send. After broadcasting, use /submit_tx with the txid._",
			{ parse_mode: "Markdown" },
		);
	}
	if (data.startsWith("status_")) {
		const escrowId = data.replace("status_", "");
		await ctx.answerCallbackQuery();
		const botCmd = bot;
		return bot.api.sendMessage(ctx.from.id, `Use /status ${escrowId} to check.`);
	}

	return ctx.answerCallbackQuery("Unknown action");
});

async function showDisputeStep(ctx) {
	const keyboard = new InlineKeyboard()
		.text("Standard", "dispute_standard")
		.text("Mediator", "dispute_mediator")
		.row()
		.text("Jury", "dispute_jury");
	await ctx.editMessageText(
		"🔄 *Create Escrow - Step 4/5*\n\nDispute mode:\n\n• Standard — timeout refund\n• Mediator — specific mediator resolves\n• Jury — community vote",
		{ parse_mode: "Markdown", reply_markup: keyboard },
	);
}

async function showReviewStep(ctx) {
	const conv = getConv(ctx.from.id);
	if (!conv) return;
	const data = conv.data;
	const amount = data.amount;
	const timeoutDays = data.timeoutDays;
	const seller = data.seller || "None (open escrow)";
	const fee = (amount / 200).toFixed(2);
	const net = (amount - parseFloat(fee)).toFixed(2);

	const keyboard = new InlineKeyboard()
		.text("✅ Confirm & Sign", "create_confirm")
		.row()
		.text("❌ Cancel", "create_cancel");

	await ctx.editMessageText(
		"📋 *Escrow Summary — Review*\n\n" +
			`Amount: **${amount} KAS**\n` +
			`Fee (0.5%): **${fee} KAS**\n` +
			`Net to seller: **${net} KAS**\n` +
			`Seller: \`${seller}\`\n` +
			`Timeout: ${timeoutDays} day(s)\n` +
			`Dispute mode: ${data.disputeMode}\n\n` +
			"_Tap Confirm to generate the covenant address and sign with your wallet._",
		{ parse_mode: "Markdown", reply_markup: keyboard },
	);
}

async function handleCreateConfirm(ctx, data) {
	const address = data.address;
	const amount = data.amount;
	const seller = data.seller || address;
	const timeoutDays = data.timeoutDays;
	const timeoutSeconds = Math.round(timeoutDays * 86400);
	const disputeMode = data.disputeMode;
	const now = Math.floor(Date.now() / 1000);

	try {
		await ctx.editMessageText(
			"⏳ *Creating escrow...*\n\nCompiling covenant address...",
			{ parse_mode: "Markdown" },
		);

		// 1. Compile the covenant to get the address
		const compileResult = await api.compileEscrow({
			buyerKey: address,
			sellerKey: seller,
			tradeHash: "0000000000000000000000000000000000000000000000000000000000000000",
			timeout: (now + timeoutSeconds).toString(),
			treasuryKey: "0000000000000000000000000000000000000000000000000000000000000000",
		});

		const covenantAddress = compileResult.covenant_address;
		if (!covenantAddress) {
			throw new Error("Covenant compilation failed");
		}

		// 2. Generate kaspa: URI for the wallet
		const sompiAmount = Math.round(amount * 100_000_000);
		const kaspaUri = `kaspa:${covenantAddress}?amount=${(sompiAmount / 100_000_000).toFixed(8)}`;

		// 3. Store pending escrow data
		updateConv(ctx.from.id, "covenantAddress", covenantAddress);
		updateConv(ctx.from.id, "sompiAmount", sompiAmount);
		updateConv(ctx.from.id, "compileResult", compileResult);
		advanceConv(ctx.from.id, "sign");

		const keyboard = new InlineKeyboard()
			.url("💳 Open in Kaspium", kaspaUri)
			.row()
			.text("📋 Copy TX Data", "sign_copy")
			.row()
			.text("✅ I've sent the TX", "sign_done")
			.row()
			.text("❌ Cancel", "create_cancel");

		await ctx.editMessageText(
			"📤 *Step 6 — Sign & Broadcast*\n\n" +
				"Send KAS to the covenant address using your wallet:\n\n" +
				`📬 Covenant address: \`${covenantAddress}\`\n` +
				`💰 Amount: **${amount} KAS**\n\n` +
				"1️⃣ Tap **Open in Kaspium** to send (or copy the address manually)\n" +
				"2️⃣ After broadcasting, tap **I've sent the TX**\n" +
				"3️⃣ Paste the transaction ID (txid) from your wallet\n\n" +
				"_Don't have Kaspium? Copy the address and send from any Kaspa wallet._",
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
	} catch (err) {
		await ctx.editMessageText("❌ Error: " + err.message + "\n\nUse /create to try again.");
		endConv(ctx.from.id);
	}
}

// ── Settlement handlers ─────────────────────────────────────────────

async function handleSettle(ctx, escrowId) {
	try {
		const data = await api.getEscrow(escrowId);
		if (data.status !== "active") {
			return await ctx.reply(
				`❌ Escrow \`${escrowId}\` is \`${data.status}\`. Only active escrows can be settled.`,
				{ parse_mode: "Markdown" },
			);
		}
		const amount = (data.amount_sompi / 1e8).toFixed(2);
		const keyboard = new InlineKeyboard()
			.text("✅ Confirm Settle", `settle_confirm_${escrowId}`)
			.row()
			.text("❌ Cancel", "create_cancel");

		await ctx.reply(
			`📤 *Settle Escrow*\n\n` +
				`ID: \`${escrowId}\`\n` +
				`Amount: ${amount} KAS\n` +
				`Fee (0.5%): ${(amount / 200).toFixed(4)} KAS\n` +
				`\nSettling releases funds to the seller. Confirm to proceed.`,
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
}

async function handleSettleConfirm(ctx, escrowId) {
	try {
		const address = getUserAddress(ctx.from.id);
		if (!address) {
			return await ctx.reply("Set your address first: /setaddress <kaspa:...>");
		}

		// Get the escrow to build the signature message
		const escrow = await api.getEscrow(escrowId);
		const message = `settle:${escrowId}:${Math.floor(Date.now() / 1000)}:${Math.random().toString(36).slice(2, 10)}`;

		await ctx.editMessageText(
			"📤 *Settlement Instructions*\n\n" +
				`Escrow: \`${escrowId}\`\n` +
				`Amount: ${(escrow.amount_sompi / 1e8).toFixed(2)} KAS\n\n` +
				"To settle, sign this message with your Kaspa wallet and paste the signature:\n\n" +
				`Message: \`${message}\`\n\n` +
				"_In Kaspium: Tools → Sign Message_\n" +
				"_In KasWare: Use the signing feature in settings_\n\n" +
				"After signing, reply with:\n`/submit_sig ${escrowId} <your-signature>`",
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
}

async function handleRefund(ctx, escrowId) {
	try {
		const data = await api.getEscrow(escrowId);
		if (data.status !== "active" && data.status !== "pending_confirmation") {
			return await ctx.reply(
				`❌ Escrow \`${escrowId}\` is \`${data.status}\`. Cannot refund.`,
				{ parse_mode: "Markdown" },
			);
		}
		const amount = (data.amount_sompi / 1e8).toFixed(2);
		const keyboard = new InlineKeyboard()
			.text("↩️ Confirm Refund", `refund_confirm_${escrowId}`)
			.row()
			.text("❌ Cancel", "create_cancel");

		await ctx.reply(
			`↩️ *Refund Escrow*\n\n` +
				`ID: \`${escrowId}\`\n` +
				`Amount: ${amount} KAS\n` +
				`\nRefunding returns the funds to the buyer. Confirm to proceed.`,
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
}

async function handleRefundConfirm(ctx, escrowId) {
	try {
		const address = getUserAddress(ctx.from.id);
		if (!address) {
			return await ctx.reply("Set your address first: /setaddress <kaspa:...>");
		}

		const escrow = await api.getEscrow(escrowId);
		const message = `refund:${escrowId}:${Math.floor(Date.now() / 1000)}:${Math.random().toString(36).slice(2, 10)}`;

		await ctx.editMessageText(
			"↩️ *Refund Instructions*\n\n" +
				`Escrow: \`${escrowId}\`\n` +
				`Amount: ${(escrow.amount_sompi / 1e8).toFixed(2)} KAS\n\n` +
				"To refund, sign this message with your Kaspa wallet and paste the signature:\n\n" +
				`Message: \`${message}\`\n\n` +
				"_In Kaspium: Tools → Sign Message_\n" +
				"After signing, reply with:\n`/submit_sig ${escrowId} <your-signature>`",
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Error: " + err.message);
	}
}

async function handleCancelEscrow(ctx, escrowId) {
	try {
		const result = await api.cancelEscrow(escrowId);
		await ctx.reply(
			`🛑 *Escrow Cancelled*\n\n` + `ID: \`${result.escrow_id}\``,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		await ctx.reply("❌ Could not cancel: " + err.message);
	}
}

// ── Handle post-broadcast TX ID ─────────────────────────────────────

async function handleBroadcastComplete(ctx, escrowId) {
	try {
		const data = await api.getEscrow(escrowId);
		const amount = (data.amount_sompi / 1e8).toFixed(2);

		const keyboard = new InlineKeyboard()
			.text("📊 View Status", `status_${escrowId}`)
			.row()
			.url("🔗 View on Explorer", `https://kas.fyi/transaction/${data.lock_tx_id}`);

		await ctx.reply(
			`✅ *Escrow Created!*\n\n` +
				`ID: \`${data.id}\`\n` +
				`Amount: ${amount} KAS\n` +
				`Status: \`${data.status}\`\n` +
				`\nShare this escrow ID with your counterparty:\n` +
				`\`${data.id}\``,
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
	} catch (err) {
		await ctx.reply("❌ Error fetching escrow: " + err.message);
	}
}

// ── Handle TX ID submission ─────────────────────────────────────────

bot.command("submit_tx", async (ctx) => {
	const parts = (ctx.match || "").trim().split(/\s+/);
	const escrowId = parts[0];
	const txId = parts[1];

	if (!escrowId || !txId) {
		return await ctx.reply("Usage: /submit_tx <escrow-id> <txid-hex>");
	}

	const conv = getConv(ctx.from.id);
	if (!conv || conv.step !== "sign") {
		return await ctx.reply("No pending escrow creation. Use /create to start.");
	}

	try {
		const data = conv.data;
		await ctx.reply("⏳ Registering escrow...", { parse_mode: "Markdown" });

		const escrow = await api.createEscrow({
			lock_tx_id: txId,
			lock_tx_output_index: 0,
			buyer_address: data.address,
			seller_address: data.seller || undefined,
			amount_sompi: data.sompiAmount,
			dispute_mode: data.disputeMode,
		});

		endConv(ctx.from.id);
		return handleBroadcastComplete(ctx, escrow.id);
	} catch (err) {
		await ctx.reply("❌ Error registering escrow: " + err.message);
	}
});

// ── Handle signature submission for settle/refund ───────────────────

bot.command("submit_sig", async (ctx) => {
	const parts = (ctx.match || "").trim().split(/\s+/);
	const escrowId = parts[0];
	const signature = parts[1];

	if (!escrowId || !signature) {
		return await ctx.reply("Usage: /submit_sig <escrow-id> <signature-hex>");
	}

	try {
		const address = getUserAddress(ctx.from.id);
		if (!address) {
			return await ctx.reply("Set your address first: /setaddress <kaspa:...>");
		}

		// We need to determine which action (settle or refund) based on context
		// For now, try settle first since it's the more common action
		await ctx.reply("⏳ Processing... This may take a moment.", { parse_mode: "Markdown" });

		// Try to settle with the signature
		const result = await api.settleEscrow(escrowId, {
			address,
			signature,
			message: `settle:${escrowId}`,
		});

		await ctx.reply(
			`✅ *Escrow Settled!*\n\n` +
				`ID: \`${result.escrow_id}\`\n` +
				`Status: \`${result.status}\`\n\n` +
				`Use /receipt ${escrowId} to view the receipt.`,
			{ parse_mode: "Markdown" },
		);
	} catch (err) {
		// If settle failed, try refund
		try {
			const result = await api.refundEscrow(escrowId, {
				address: getUserAddress(ctx.from.id),
				signature,
				message: `refund:${escrowId}`,
			});
			await ctx.reply(
				`↩️ *Escrow Refunded!*\n\n` +
					`ID: \`${result.escrow_id}\`\n` +
					`Status: \`${result.status}\``,
				{ parse_mode: "Markdown" },
			);
		} catch {
			await ctx.reply("❌ Error: " + err.message);
		}
	}
});

// ── Claim handler ────────────────────────────────────────────────────

async function handleClaim(ctx, escrowId) {
	try {
		const data = await api.getEscrow(escrowId);
		const amount = (data.amount_sompi / 1e8).toFixed(2);

		const keyboard = new InlineKeyboard().url(
			"🔓 Claim via Browser",
			`https://daglock.com`,
		);

		await ctx.reply(
			`🔓 *Claim Escrow*\n\n` +
				`You have been offered an escrow:\n\n` +
				`Amount: ${amount} KAS\n` +
				`Escrow: \`${escrowId}\`\n\n` +
				`To claim, open in browser and sign with your wallet:`,
			{ parse_mode: "Markdown", reply_markup: keyboard },
		);
	} catch (err) {
		await ctx.reply("❌ Could not load escrow: " + err.message);
	}
}

// ── Error handler ────────────────────────────────────────────────────

bot.catch((err) => {
	console.error("Bot error:", err);
});

// ── Start the bot ────────────────────────────────────────────────────

bot.start({ drop_pending_updates: true });
console.log(`DagLock Bot running... (indexer: ${apiUrl})`);
