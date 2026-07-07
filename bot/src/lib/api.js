// DagLock Indexer REST API client for the Telegram bot.
// Features: request timeout (10s), retry with exponential backoff (3 attempts).

const MAX_RETRIES = 3;
const REQUEST_TIMEOUT_MS = 10_000;
const RETRY_DELAYS_MS = [1_000, 2_000, 4_000]; // exponential: 1s, 2s, 4s

function sleep(ms) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export class ApiClient {
	constructor(baseUrl) {
		this.baseUrl = baseUrl;
	}

	async request(path, options = {}, auth) {
		const headers = { "Content-Type": "application/json", ...options.headers };
		if (auth) {
			headers["X-Daglock-Address"] = auth.address;
			headers["X-Daglock-Signature"] = auth.signature;
			headers["X-Daglock-Message"] = auth.message;
		}
		const url = `${this.baseUrl}/v1${path}`;

		// Determine if retry is safe (GET = idempotent, POST = not)
		const method = (options.method || "GET").toUpperCase();
		const isIdempotent =
			method === "GET" || method === "HEAD" || method === "OPTIONS";

		for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
			const controller = new AbortController();
			const timeoutId = setTimeout(
				() => controller.abort(),
				REQUEST_TIMEOUT_MS,
			);

			try {
				const res = await fetch(url, {
					headers,
					signal: controller.signal,
					...options,
				});
				clearTimeout(timeoutId);

				if (res.ok) {
					return res.json();
				}

				// Non-2xx — parse error, retry on 5xx if idempotent
				const errBody = await res
					.json()
					.catch(() => ({ error: { message: res.statusText } }));
				const errMsg = errBody.error?.message || `HTTP ${res.status}`;

				// Retry on 5xx (server errors) and 429 (rate limit) for idempotent requests
				const shouldRetry =
					(res.status >= 500 || res.status === 429) &&
					isIdempotent &&
					attempt < MAX_RETRIES;

				if (shouldRetry) {
					const delay = RETRY_DELAYS_MS[attempt] || 5_000;
					console.warn(
						`[ApiClient] Retry ${attempt + 1}/${MAX_RETRIES} for ${url} (${res.status}) — waiting ${delay}ms`,
					);
					await sleep(delay);
					continue;
				}

				throw new Error(errMsg);
			} catch (err) {
				clearTimeout(timeoutId);

				// Network or abort errors are retryable for idempotent requests
				const isNetworkError =
					err.name === "AbortError" || err.type === "system" || !err.status;
				const shouldRetry =
					isNetworkError && isIdempotent && attempt < MAX_RETRIES;

				if (shouldRetry) {
					const delay = RETRY_DELAYS_MS[attempt] || 5_000;
					console.warn(
						`[ApiClient] Retry ${attempt + 1}/${MAX_RETRIES} for ${url} (network error) — waiting ${delay}ms`,
					);
					await sleep(delay);
					continue;
				}

				throw err;
			}
		}
	}

	// ── Escrow endpoints ──────────────────────────────────────────────

	getEscrow(id) {
		return this.request(`/escrows/${encodeURIComponent(id)}`);
	}

	listEscrows(address, { role, status, limit, offset } = {}) {
		let path = `/escrows?address=${encodeURIComponent(address)}`;
		if (role) path += `&role=${role}`;
		if (status) path += `&status=${status}`;
		if (limit) path += `&limit=${limit}`;
		if (offset) path += `&offset=${offset}`;
		return this.request(path);
	}

	createEscrow(data) {
		return this.request("/escrows", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	settleEscrow(id) {
		return this.request(`/escrows/${encodeURIComponent(id)}/settle`, {
			method: "POST",
		});
	}

	refundEscrow(id) {
		return this.request(`/escrows/${encodeURIComponent(id)}/refund`, {
			method: "POST",
		});
	}

	disputeEscrow(id, reason) {
		return this.request(`/escrows/${encodeURIComponent(id)}/dispute`, {
			method: "POST",
			body: JSON.stringify({ reason }),
		});
	}

	cancelEscrow(id) {
		return this.request(`/escrows/${encodeURIComponent(id)}/cancel`, {
			method: "POST",
		});
	}

	swapEscrow(id, preimage) {
		return this.request(`/escrows/${encodeURIComponent(id)}/swap`, {
			method: "POST",
			body: JSON.stringify({ preimage }),
		});
	}

	// ── Offer endpoints ───────────────────────────────────────────────

	listOffers({ asset, side, status } = {}) {
		let path = "/offers?";
		if (asset) path += `asset=${asset}&`;
		if (side) path += `side=${side}&`;
		if (status) path += `status=${status}&`;
		return this.request(path);
	}

	createOffer(data) {
		return this.request("/offers", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	acceptOffer(id, counterpartyAddress) {
		return this.request(`/offers/${encodeURIComponent(id)}/accept`, {
			method: "POST",
			body: JSON.stringify({ counterparty_address: counterpartyAddress }),
		});
	}

	cancelOffer(id) {
		return this.request(`/offers/${encodeURIComponent(id)}/cancel`, {
			method: "POST",
		});
	}

	// ── Reputation ────────────────────────────────────────────────────

	getReputation(address) {
		return this.request(`/reputation/${encodeURIComponent(address)}`);
	}

	// ── Receipts ──────────────────────────────────────────────────────

	getReceipt(id) {
		return this.request(`/receipts/${encodeURIComponent(id)}`);
	}

	generateSwap() {
		return this.request("/swap/generate");
	}

	// ── Vaults ────────────────────────────────────────────────────────

	listVaults(owner) {
		return this.request(`/vaults?owner=${encodeURIComponent(owner)}`);
	}

	getVault(id) {
		return this.request(`/vaults/${encodeURIComponent(id)}`);
	}

	createVault(data, auth) {
		return this.request(
			"/vaults",
			{
				method: "POST",
				body: JSON.stringify(data),
			},
			auth,
		);
	}

	withdrawVault(id, ownerAddress, signature, auth) {
		return this.request(
			`/vaults/${encodeURIComponent(id)}/withdraw`,
			{
				method: "POST",
				body: JSON.stringify({ owner_address: ownerAddress, signature }),
			},
			auth,
		);
	}

	// ── Messages ──────────────────────────────────────────────────────

	sendMessage(escrowId, content, auth) {
		return this.request(
			`/escrows/${encodeURIComponent(escrowId)}/messages`,
			{
				method: "POST",
				body: JSON.stringify({ content }),
			},
			auth,
		);
	}

	listMessages(escrowId, auth) {
		return this.request(
			`/escrows/${encodeURIComponent(escrowId)}/messages`,
			undefined,
			auth,
		);
	}

	// ── Evidence ──────────────────────────────────────────────────────

	listEvidence(escrowId) {
		return this.request(`/escrows/${encodeURIComponent(escrowId)}/evidence`);
	}

	// ── Fees ──────────────────────────────────────────────────────────

	feeEstimate(amountKas) {
		return this.request(`/fees/estimate?amount_kas=${encodeURIComponent(amountKas)}`);
	}

	// ── Blocklist ─────────────────────────────────────────────────────

	blockUser(blockedAddress, reason, auth) {
		return this.request(
			"/blocks",
			{
				method: "POST",
				body: JSON.stringify({ blocked_address: blockedAddress, reason }),
			},
			auth,
		);
	}

	unblockUser(blockId, auth) {
		return this.request(`/blocks/${encodeURIComponent(blockId)}`, {
			method: "POST",
		}, auth);
	}

	listBlocks(address, auth) {
		return this.request(`/blocks?address=${encodeURIComponent(address)}`, undefined, auth);
	}

	// ── Reports ───────────────────────────────────────────────────────

	reportUser(reportedAddress, reason, escrowId, auth) {
		return this.request(
			"/reports",
			{
				method: "POST",
				body: JSON.stringify({ reported_address: reportedAddress, reason, escrow_id: escrowId }),
			},
			auth,
		);
	}

	// ── Trade Feedback ────────────────────────────────────────────────

	submitFeedback(escrowId, rating, comment, auth) {
		return this.request(
			`/escrows/${encodeURIComponent(escrowId)}/feedback`,
			{
				method: "POST",
				body: JSON.stringify({ rating, comment }),
			},
			auth,
		);
	}

	// ── Milestones ────────────────────────────────────────────────────

	listMilestones(address) {
		return this.request(`/milestones?address=${encodeURIComponent(address)}`);
	}

	getMilestone(id) {
		return this.request(`/milestones/${encodeURIComponent(id)}`);
	}

	createMilestone(data) {
		return this.request("/milestones", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	releaseMilestone(id) {
		return this.request(`/milestones/${encodeURIComponent(id)}/release`, {
			method: "POST",
		});
	}

	listFeedback(escrowId) {
		return this.request(`/escrows/${encodeURIComponent(escrowId)}/feedback`);
	}

	// ── Multi-party escrows ───────────────────────────────────────────

	listMultiEscrows(address) {
		return this.request(`/multi-escrows?address=${encodeURIComponent(address)}`);
	}

	getMultiEscrow(id) {
		return this.request(`/multi-escrows/${encodeURIComponent(id)}`);
	}

	createMultiEscrow(data) {
		return this.request("/multi-escrows", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	signMultiEscrow(id, address) {
		return this.request(`/multi-escrows/${encodeURIComponent(id)}/sign`, {
			method: "POST",
			body: JSON.stringify({ address }),
		});
	}

	refundMultiEscrow(id) {
		return this.request(`/multi-escrows/${encodeURIComponent(id)}/refund`, {
			method: "POST",
		});
	}

	swapMultiEscrow(id) {
		return this.request(`/multi-escrows/${encodeURIComponent(id)}/swap`, {
			method: "POST",
		});
	}

	// ── AI Mediation ──────────────────────────────────────────────────

	mediateEscrow(id, buyerClaim, sellerClaim, auth) {
		return this.request(`/escrows/${encodeURIComponent(id)}/mediate`, {
			method: "POST",
			body: JSON.stringify({ buyer_claim: buyerClaim, seller_claim: sellerClaim }),
		}, auth);
	}

	acceptMediation(id, party, auth) {
		return this.request(`/escrows/${encodeURIComponent(id)}/mediate/${party}/accept`, {
			method: "POST",
			body: JSON.stringify({ accept: true }),
		}, auth);
	}

	getMediation(id) {
		return this.request(`/escrows/${encodeURIComponent(id)}/mediate`);
	}

	// ── Compile ───────────────────────────────────────────────────────

	compileEscrow(params) {
		return this.request("/compile", {
			method: "POST",
			body: JSON.stringify({ template: "daglock", params }),
		});
	}

	// ── Network / Explorer ────────────────────────────────────────────

	getExplorerUrl() {
		return this.request("/network/explorer");
	}

	// ── Counter-offers ────────────────────────────────────────────────

	// ── offers API (with auth) ────────────────────────────────────────

	makeAuth(address, message) {
		// Mock auth for testnet — HMAC-SHA256 of the message
		const crypto = require("crypto");
		const msgHash = crypto.createHash("sha256").update(message).digest();
		const key = address.padEnd(64, "0").slice(0, 64);
		const sig = crypto.createHmac("sha256", Buffer.from(key, "hex")).update(msgHash).digest();
		const fullSig = Buffer.concat([sig, msgHash.slice(0, 32)]).toString("hex");
		return {
			"X-Daglock-Address": address,
			"X-Daglock-Signature": fullSig,
			"X-Daglock-Message": message,
		};
	}

	counterOffer(offerId, amountSompi, message, address) {
		const msg = `counter_offer:${offerId}:${Math.floor(Date.now() / 1000)}`;
		return this.request(
			`/offers/${encodeURIComponent(offerId)}/counter`,
			{
				method: "POST",
				body: JSON.stringify({ amount_sompi: amountSompi, message }),
				headers: this.makeAuth(address, msg),
			},
		);
	}

	listCounters(offerId) {
		return this.request(`/offers/${encodeURIComponent(offerId)}/counters`);
	}

	// ── Stats & Health ────────────────────────────────────────────────

	getStats() {
		return this.request("/stats");
	}

	getHealth() {
		return this.request("/health");
	}
}
