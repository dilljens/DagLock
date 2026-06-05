// DagLock Indexer REST API client for the Telegram bot.

export class ApiClient {
	constructor(baseUrl) {
		this.baseUrl = baseUrl;
	}

	async request(path, options = {}) {
		const url = `${this.baseUrl}/v1${path}`;
		const res = await fetch(url, {
			headers: { "Content-Type": "application/json", ...options.headers },
			...options,
		});
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

	// ── Swap ─────────────────────────────────────────────────────────

	swapEscrow(id, preimage) {
		return this.request(`/escrows/${encodeURIComponent(id)}/swap`, {
			method: "POST",
			body: JSON.stringify({ preimage }),
		});
	}

	// ── Vaults ────────────────────────────────────────────────────────

	listVaults(owner) {
		return this.request(`/vaults?owner=${encodeURIComponent(owner)}`);
	}

	getVault(id) {
		return this.request(`/vaults/${encodeURIComponent(id)}`);
	}

	createVault(data) {
		return this.request("/vaults", {
			method: "POST",
			body: JSON.stringify(data),
		});
	}

	withdrawVault(id, ownerAddress, signature) {
		return this.request(`/vaults/${encodeURIComponent(id)}/withdraw`, {
			method: "POST",
			body: JSON.stringify({ owner_address: ownerAddress, signature }),
		});
	}

	// ── Messages ──────────────────────────────────────────────────────

	sendMessage(escrowId, content) {
		return this.request(`/escrows/${encodeURIComponent(escrowId)}/messages`, {
			method: "POST",
			body: JSON.stringify({ content }),
		});
	}

	listMessages(escrowId) {
		return this.request(`/escrows/${encodeURIComponent(escrowId)}/messages`);
	}

	// ── Stats & Health ────────────────────────────────────────────────

	getStats() {
		return this.request("/stats");
	}

	getHealth() {
		return this.request("/health");
	}
}
