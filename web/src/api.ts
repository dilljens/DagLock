export type Health = {
	status: string;
	version: string;
	node_synced: boolean;
	node_daa_score: number;
	uptime_seconds: number;
};

export type NetworkInfo = {
	network: string;
	daa_score: number;
	block_count: number;
	difficulty: number;
	bps: number;
	daglock_kas_template_hash?: string | null;
	daglock_krc20_template_hash?: string | null;
};

export type Stats = {
	total_escrows: number;
	active_escrows: number;
	disputed_escrows: number;
	settled_escrows: number;
	refunded_escrows: number;
	cancelled_escrows: number;
	total_volume_kas: string;
	total_fees_collected_kas: string;
	unique_buyers: number;
	unique_sellers: number;
};

export type Offer = {
	id: string;
	creator_address: string;
	side: string;
	base_asset: string;
	quote_asset: string;
	amount_sompi: number;
	counterparty_address?: string | null;
	status: string;
	expires_at?: number | null;
	created_at: number;
};

export type CreateOfferRequest = {
	creator_address: string;
	side: string;
	base_asset: string;
	quote_asset: string;
	amount_sompi: number;
	counterparty_address?: string;
	expires_at?: number;
};

export type Escrow = {
	id: string;
	lock_tx_id: string;
	lock_tx_output_index: number;
	status: string;
	asset_type: string;
	buyer_address: string;
	seller_address?: string | null;
	amount_sompi: number;
	fee_sompi: number;
	template_hash: number[];
	expiration_daa_score?: number | null;
	disputed_at?: number | null;
	dispute_reason?: string | null;
	cancelled_at?: number | null;
	expired_at?: number | null;
	created_at: number;
	settled_at?: number | null;
	refunded_at?: number | null;
	mediator_key?: string | null;
	dispute_outcome?: string | null;
	dispute_resolved_at?: number | null;
};

export type CreateEscrowRequest = {
	lock_tx_id: string;
	lock_tx_output_index: number;
	buyer_address: string;
	seller_address?: string;
	amount_sompi: number;
	expiration_daa_score?: number;
	asset_type?: string;
	mediator_key?: string;
};

export type AuthHeaders = {
	address: string;
	signature: string;
	message: string;
};

export type Reputation = {
	address: string;
	trade_count: number;
	total_volume_sompi: number;
	settled_count: number;
	refunded_count: number;
	disputed_count: number;
	first_trade_at?: number | null;
	age_days: number;
	dispute_rate: number;
	refund_rate: number;
	score: number;
};

export type Receipt = {
	receipt_id: string;
	escrow_id: string;
	status: string;
	asset: string;
	amount_sompi: number;
	fee_sompi: number;
	buyer_address: string;
	seller_address?: string | null;
	lock_tx_id: string;
	lock_tx_output_index: number;
	expiration_daa_score?: number | null;
	disputed_at?: number | null;
	dispute_reason?: string | null;
	cancelled_at?: number | null;
	expired_at?: number | null;
	settled_at?: number | null;
	refunded_at?: number | null;
};

export type DisputeEvidence = {
	id: string;
	escrow_id: string;
	submitted_by: string;
	content: string;
	content_hash: string;
	signed_message?: string | null;
	created_at: number;
};

async function loadJson<T>(path: string): Promise<T> {
	const response = await fetch(path);
	if (!response.ok) {
		const body = await response.text();
		throw new Error(body);
	}
	return response.json() as Promise<T>;
}

async function postJson<T>(
	path: string,
	body: unknown,
	auth?: AuthHeaders,
): Promise<T> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
	};
	if (auth) {
		headers["X-Daglock-Address"] = auth.address;
		headers["X-Daglock-Signature"] = auth.signature;
		headers["X-Daglock-Message"] = auth.message;
	}
	const response = await fetch(path, {
		method: "POST",
		headers,
		body: JSON.stringify(body),
	});
	if (!response.ok) {
		const text = await response.text();
		throw new Error(text);
	}
	return response.json() as Promise<T>;
}

async function postEmpty<T>(path: string, auth?: AuthHeaders): Promise<T> {
	const headers: Record<string, string> = {};
	if (auth) {
		headers["X-Daglock-Address"] = auth.address;
		headers["X-Daglock-Signature"] = auth.signature;
		headers["X-Daglock-Message"] = auth.message;
	}
	const response = await fetch(path, {
		method: "POST",
		headers,
	});
	if (!response.ok) {
		const text = await response.text();
		throw new Error(text);
	}
	return response.json() as Promise<T>;
}

export const api = {
	health: () => loadJson<Health>("/v1/health"),
	network: () => loadJson<NetworkInfo>("/v1/network"),
	stats: () => loadJson<Stats>("/v1/stats"),

	// Escrows
	escrows: (address: string) =>
		loadJson<{ escrows: Escrow[]; total: number }>(
			`/v1/escrows?address=${encodeURIComponent(address)}`,
		),
	escrow: (id: string) =>
		loadJson<Escrow>(`/v1/escrows/${encodeURIComponent(id)}`),
	createEscrow: (req: CreateEscrowRequest) =>
		postJson<Escrow>("/v1/escrows", req),
	settleEscrow: (id: string, auth: AuthHeaders) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/settle`,
			auth,
		),
	refundEscrow: (id: string, auth: AuthHeaders) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/refund`,
			auth,
		),
	disputeEscrow: (id: string, reason: string) =>
		postJson<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/dispute`,
			{ reason },
		),
	cancelEscrow: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/cancel`,
		),

	// Offers
	offers: () =>
		loadJson<{ offers: Offer[]; total: number }>("/v1/offers?status=proposed"),
	createOffer: (req: CreateOfferRequest) => postJson<Offer>("/v1/offers", req),
	acceptOffer: (id: string, counterparty_address: string) =>
		postJson<{ status: string; offer_id: string }>(
			`/v1/offers/${encodeURIComponent(id)}/accept`,
			{ counterparty_address },
		),
	cancelOffer: (id: string) =>
		postEmpty<{ status: string; offer_id: string }>(
			`/v1/offers/${encodeURIComponent(id)}/cancel`,
		),

	// Lookups
	reputation: (address: string) =>
		loadJson<Reputation>(`/v1/reputation/${encodeURIComponent(address)}`),
	receipt: (id: string) =>
		loadJson<Receipt>(`/v1/receipts/${encodeURIComponent(id)}`),

	// Evidence
	submitEvidence: (escrowId: string, content: string, signedMessage?: string, auth?: AuthHeaders) =>
		postJson<DisputeEvidence>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/evidence`,
			{ content, signed_message: signedMessage },
			auth,
		),
	listEvidence: (escrowId: string) =>
		loadJson<{ evidence: DisputeEvidence[]; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/evidence`,
		),
	resolveDispute: (escrowId: string, outcome: string, resolved_by: string, auth?: AuthHeaders) =>
		postJson<{ status: string; escrow_id: string; outcome: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/resolve-dispute`,
			{ outcome, resolved_by },
			auth,
		),
};
