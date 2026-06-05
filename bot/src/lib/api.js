// DagLock Indexer REST API client for the Telegram bot.

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
		const res = await fetch(url, { headers, ...options });
		if (!res.ok) {
			const err = await res
				.json()
				.catch(() => ({ error: { message: res.statusText } }));
			throw new Error(err.error?.message || `HTTP ${res.status}`);
		}
		return res.json();
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

	// ── Stats & Health ────────────────────────────────────────────────

	getStats() {
		return this.request("/stats");
	}

	getHealth() {
		return this.request("/health");
	}
}
