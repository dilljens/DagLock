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
	{ command: "list", description: "List your escrows" },
	{ command: "offers", description: "Browse open offers" },
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
	{ command: "help", description: "Show help" },
]);

bot.command("start", async (ctx) => {
	const payload = ctx.match;
	if (payload?.startsWith("claim_")) {
		const escrowId = payload.replace("claim_", "");
		return handleClaim(ctx, escrowId);
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
			"/cancel <id> — Cancel escrow\n" +
			"/reputation <address> — Check reputation\n" +
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
		"*Create Escrow - Step 1/4*\n\nHow much KAS do you want to escrow?\n\nExample: `100` or `5000.5`",
		{ parse_mode: "Markdown" },
	);
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
			msg += `  ID: \`${o.id}\`\n\n`;
		}
		if (offers.length > 5) msg += `_...and ${offers.length - 5} more_\n`;
		msg += "💡 Use /status <id> for details";

		await ctx.reply(msg, { parse_mode: "Markdown" });
	} catch (err) {
		await ctx.reply("❌ Could not fetch offers: " + err.message);
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
		await ctx.reply(
			`⚠️ *Escrow Disputed*\n\n` +
				`ID: \`${result.escrow_id}\`\n` +
				`Reason: ${reason}`,
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
			msg += `[${date}] \`${sender}...\`: ${m.content}\n`;
		}
		if (msgs.length > 5)
			msg += `\n_...${msgs.length - 5} older messages hidden_`;

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

// ── Help command ────────────────────────────────────────────────────

// --- Wizard text handler ---
bot.on("message:text", async (ctx) => {
	const conv = getConv(ctx.from.id);
	if (!conv) return;

	const text = ctx.message.text.trim();
	const step = conv.step;

	try {
		if (step === "amount") {
			const num = parseFloat(text);
			if (isNaN(num) || num <= 0) {
				return await ctx.reply("Enter a valid amount, e.g. `100`", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "amount", num);
			advanceConv(ctx.from.id, "counterparty");
			return await ctx.reply(
				"*Create Escrow - Step 2/4*\n\nEnter the seller's Kaspa address:\n\nOr type `skip` to create without a seller.",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "counterparty") {
			if (text !== "skip" && !text.startsWith("kaspa:")) {
				return await ctx.reply("Enter a `kaspa:` address or `skip`", {
					parse_mode: "Markdown",
				});
			}
			if (text !== "skip") updateConv(ctx.from.id, "seller", text);
			advanceConv(ctx.from.id, "timeout");
			return await ctx.reply(
				"*Create Escrow - Step 3/4*\n\nChoose a timeout:\n\n`1` - 24 hours\n`3` - 3 days\n`7` - 7 days\n`30` - 30 days",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "timeout") {
			const days = parseInt(text);
			if (isNaN(days) || ![1, 3, 7, 30].includes(days)) {
				return await ctx.reply("Enter `1`, `3`, `7`, or `30`", {
					parse_mode: "Markdown",
				});
			}
			updateConv(ctx.from.id, "timeoutDays", days);
			advanceConv(ctx.from.id, "dispute");
			return await ctx.reply(
				"*Create Escrow - Step 4/4*\n\nDispute mode:\n\n`standard` - Timeout refund\n`mediator` - Specific mediator\n`jury` - Community vote",
				{ parse_mode: "Markdown" },
			);
		}

		if (step === "dispute") {
			const mode = text.toLowerCase();
			if (!["standard", "mediator", "jury"].includes(mode)) {
				return await ctx.reply("Choose `standard`, `mediator`, or `jury`", {
					parse_mode: "Markdown",
				});
			}
			const data = conv.data;
			endConv(ctx.from.id);

			// Redirect to web app — the bot cannot sign transactions
			const webUrl = process.env.WEB_URL || "https://daglock.com";
			const params = new URLSearchParams({
				amount: data.amount.toString(),
				dispute_mode: mode,
			});
			if (data.seller) params.set("seller", data.seller);
			const redirectUrl = `${webUrl}/#/escrows?${params.toString()}`;

			await ctx.reply(
				"✅ *Escrow params ready!*\n\n" +
					`Amount: ${data.amount} KAS\n` +
					`Dispute: ${mode}\n` +
					(data.seller ? `Seller: \`${data.seller.slice(0, 16)}...\`\n` : "") +
					"\nTo complete the escrow, you need to:\n" +
					"1️⃣ Open the web app\n" +
					"2️⃣ Connect KasWare wallet\n" +
					"3️⃣ Sign and broadcast the covenant transaction\n\n" +
					`🔗 [Open DagLock Web App](${redirectUrl})`,
				{ parse_mode: "Markdown", link_preview_options: { is_disabled: true } },
			);
		}
	} catch (e) {
		await ctx.reply("Error: " + e.message);
		endConv(ctx.from.id);
	}
});
bot.command("help", async (ctx) => {
	await ctx.reply(
		"🔒 *DagLock Bot Help*\n\n" +
			"*Setup:*\n" +
			"/setaddress — Set your Kaspa address\n\n" +
			"*Commands:*\n" +
			"/create — Create escrow (opens web interface)\n" +
			"/claim <id> — Claim an escrow from a trade link\n" +
			"/list — List your escrows\n" +
			"/offers — Browse open offers\n" +
			"/swap <id> <hex> — Atomic swap settle via preimage\n" +
			"/vaults — List your vaults\n" +
			"/msg <id> <text> — Send a message on an escrow\n" +
			"/messages <id> — Read message thread\n" +
			"/status <id> — Check escrow status\n" +
			"/receipt <id> — Fetch receipt\n" +
			"/dispute <id> <reason> — Dispute escrow\n" +
			"/cancel <id> — Cancel escrow\n" +
			"/reputation <address> — Check reputation\n\n" +
			"*Trade Links:*\n" +
			"Share a link to let someone claim your escrow:\n" +
			"`https://t.me/DagLock_bot?start=claim_<escrow-id>`",
		{ parse_mode: "Markdown" },
	);
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
