const API_BASE = import.meta.env.VITE_API_URL || "";
console.log("[DagLock] API_BASE:", API_BASE);

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
	price_type: string;
	price_offset?: number | null;
	min_price?: number | null;
	max_price?: number | null;
	current_price?: number | null;
	price_currency: string;
	price_updated_at?: number | null;
};

export type CreateOfferRequest = {
	creator_address: string;
	side: string;
	base_asset: string;
	quote_asset: string;
	amount_sompi: number;
	counterparty_address?: string;
	expires_at?: number;
	price_type?: string;
	price_offset?: number;
	min_price?: number;
	max_price?: number;
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
	dispute_mode?: string | null;
	price_at_creation?: number | null;
	price_currency?: string | null;
	dispute_outcome?: string | null;
	dispute_resolved_at?: number | null;
	trade_hash?: string | null;
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
	dispute_mode?: string;
	price_at_creation?: number;
	price_currency?: string;
	trade_hash?: string;
	price_type?: string;
};

export type AuthHeaders = {
	address: string;
	signature: string;
	message: string;
};

export type Reputation = {
	address: string;
	trade_count: number;
	recent_trade_count: number;
	total_volume_sompi: number;
	vouches_received: number;
	vouches_given: number;
	vouch_score?: number | null;
	mediator_stats?: MediatorStats | null;
	trading_concentration: number;
	settled_count: number;
	refunded_count: number;
	disputed_count: number;
	first_trade_at?: number | null;
	age_days: number;
	dispute_rate: number;
	refund_rate: number;
	score: number;
	telegram_handle?: string | null;
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

export type JurorRegistration = {
	address: string;
	registered_at: number;
	total_cases_assigned: number;
	total_cases_voted: number;
	reliability_score: number;
};

export type JuryCase = {
	id: string;
	escrow_id: string;
	status: string;
	juror_count: number;
	threshold: number;
	votes_for_seller: number;
	votes_for_buyer: number;
	created_at: number;
	decided_at?: number | null;
	outcome?: string | null;
	jurors: string[];
};

export type JuryVote = {
	case_id: string;
	juror_address: string;
	vote: string;
	voted_at: number;
	reasoning?: string | null;
};

export type CastVoteRequest = {
	vote: string;
	reasoning?: string;
};

export type MediatorStats = {
	disputes_mediated: number;
	rulings_accepted: number;
	acceptance_rate: number;
	years_active: number;
	score: number;
};

export type EscrowMessage = {
	id: string;
	escrow_id: string;
	sender_address: string;
	content: string;
	created_at: number;
};

export type SendMessageRequest = {
	content: string;
};

export type Vouch = {
	id: string;
	voucher_address: string;
	subject_address: string;
	escrow_id?: string | null;
	note?: string | null;
	created_at: number;
	expires_at: number;
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

// ── Vault Types ─────────────────────────────────────────────────

export type VaultType =
	| "time"
	| "beneficiary"
	| "deadman"
	| "inheritance"
	| "multisig";

export type VaultStatus = "locked" | "unlocked" | "expired" | "transferred";

export type Vault = {
	id: string;
	owner_address: string;
	beneficiary_address?: string | null;
	vault_type: VaultType;
	status: VaultStatus;
	amount_sompi: number;
	timeout: number;
	lock_tx_id?: string | null;
	lock_tx_output_index?: number | null;
	created_at: number;
	unlocked_at?: number | null;
	expires_at?: number | null;
};

export type CreateVaultRequest = {
	owner_address: string;
	beneficiary_address?: string;
	vault_type: VaultType;
	amount_sompi: number;
	timeout: number;
	lock_tx_id?: string;
	lock_tx_output_index?: number;
};

async function loadAuthJson<T>(path: string, auth: AuthHeaders): Promise<T> {
	const response = await fetch(API_BASE + path, {
		headers: {
			"X-Daglock-Address": auth.address,
			"X-Daglock-Signature": auth.signature,
			"X-Daglock-Message": auth.message,
		},
	});
	if (!response.ok) {
		const body = await response.text();
		throw new Error(body);
	}
	return response.json() as Promise<T>;
}

async function loadJson<T>(path: string): Promise<T> {
	const response = await fetch(API_BASE + path);
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
	const response = await fetch(API_BASE + path, {
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
	const response = await fetch(API_BASE + path, {
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
	compile: (template: string, params: Record<string, string>) =>
		postJson<{
			script: string;
			template_hash: string;
			abi: { name: string }[];
		}>("/v1/compile", { template, params }),
	network: () => loadJson<NetworkInfo>("/v1/network"),
	networkPrice: () =>
		loadJson<{ kas_usd: number; updated_at: number }>("/v1/network/price"),
	stats: () => loadJson<Stats>("/v1/stats"),

	// Vaults
	vaults: (owner?: string) =>
		loadJson<{ vaults: Vault[]; total: number }>(
			`/v1/vaults${owner ? `?owner=${encodeURIComponent(owner)}` : ""}`,
		),
	vault: (id: string) =>
		loadJson<Vault>(`/v1/vaults/${encodeURIComponent(id)}`),
	createVault: (req: CreateVaultRequest) => postJson<Vault>("/v1/vaults", req),
	withdrawVault: (id: string, ownerAddress: string, signature: string) =>
		postJson<{ status: string; vault_id: string }>(
			`/v1/vaults/${encodeURIComponent(id)}/withdraw`,
			{ owner_address: ownerAddress, signature },
		),

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
	disputeEscrow: (id: string, reason: string, mode?: string) =>
		postJson<{ status: string; escrow_id: string; jury_case_id?: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/dispute`,
			{ reason, mode },
		),
	cancelEscrow: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/cancel`,
		),
	swapEscrow: (id: string, preimage: string) =>
		postJson<{ status: string; escrow_id: string; method: string; preimage_hash: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/swap`,
			{ preimage },
		),
	generateSwap: () =>
		loadJson<{ secret: string; hash: string }>("/v1/swap/generate"),

	// Offers
	offers: (creator?: string) =>
		loadJson<{ offers: Offer[]; total: number }>(
			`/v1/offers?status=proposed${creator ? `&creator=${encodeURIComponent(creator)}` : ""}`,
		),
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

	// Vouching
	vouch: (
		subjectAddress: string,
		auth: AuthHeaders,
		escrowId?: string,
		note?: string,
	) =>
		postJson<{ status: string; vouch: Vouch }>(
			"/v1/vouches",
			{ subject_address: subjectAddress, escrow_id: escrowId, note },
			auth,
		),
	unvouch: (vouchId: string, auth: AuthHeaders) =>
		postEmpty<{ status: string; vouch_id: string }>(
			`/v1/vouches/${encodeURIComponent(vouchId)}`,
			auth,
		),
	listVouchesBySubject: (address: string) =>
		loadJson<{ vouches: Vouch[]; total: number }>(
			`/v1/vouches?subject=${encodeURIComponent(address)}`,
		),
	listVouchesByVoucher: (address: string) =>
		loadJson<{ vouches: Vouch[]; total: number }>(
			`/v1/vouches?voucher=${encodeURIComponent(address)}`,
		),

	// Jury
	juryRegister: (auth: AuthHeaders) =>
		postJson<{ status: string; address: string }>(
			"/v1/jury/register",
			{},
			auth,
		),
	juryUnregister: (auth: AuthHeaders) =>
		postJson<{ status: string; address: string }>(
			"/v1/jury/unregister",
			{},
			auth,
		),
	juryCases: (auth: AuthHeaders) =>
		loadAuthJson<{ cases: JuryCase[]; total: number }>("/v1/jury/cases", auth),
	juryCase: (caseId: string) =>
		loadJson<JuryCase>(`/v1/jury/cases/${encodeURIComponent(caseId)}`),
	juryVote: (
		caseId: string,
		vote: string,
		reasoning?: string,
		auth?: AuthHeaders,
	) =>
		postJson<{ status: string; vote: string; verdict?: string | null }>(
			`/v1/jury/cases/${encodeURIComponent(caseId)}/vote`,
			{ vote, reasoning },
			auth,
		),
	juryCandidates: () =>
		loadJson<{ candidates: JurorRegistration[]; total: number }>(
			"/v1/jury/candidates",
		),

	// Messages
	sendMessage: (escrowId: string, content: string, auth: AuthHeaders) =>
		postJson<{ status: string; message: EscrowMessage }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/messages`,
			{ content },
			auth,
		),
	listMessages: (escrowId: string, auth: AuthHeaders) =>
		loadAuthJson<{
			messages: EscrowMessage[];
			total: number;
			escrow_id: string;
		}>(`/v1/escrows/${encodeURIComponent(escrowId)}/messages`, auth),

	// Identity
	createIdentity: (
		platform: string,
		handle: string,
		signedMessage: string,
		signatureHex: string,
		auth: AuthHeaders,
	) =>
		postJson<{
			status: string;
			address: string;
			platform: string;
			handle: string;
		}>(
			"/v1/identity",
			{
				platform,
				handle,
				signed_message: signedMessage,
				signature_hex: signatureHex,
			},
			auth,
		),
	reputation: (address: string) =>
		loadJson<Reputation>(`/v1/reputation/${encodeURIComponent(address)}`),
	receipt: (id: string) =>
		loadJson<Receipt>(`/v1/receipts/${encodeURIComponent(id)}`),

	// Evidence
	submitEvidence: (
		escrowId: string,
		content: string,
		signedMessage?: string,
		auth?: AuthHeaders,
	) =>
		postJson<DisputeEvidence>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/evidence`,
			{ content, signed_message: signedMessage },
			auth,
		),
	listEvidence: (escrowId: string) =>
		loadJson<{ evidence: DisputeEvidence[]; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/evidence`,
		),
	resolveDispute: (
		escrowId: string,
		outcome: string,
		resolved_by: string,
		auth?: AuthHeaders,
	) =>
		postJson<{ status: string; escrow_id: string; outcome: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/resolve-dispute`,
			{ outcome, resolved_by },
			auth,
		),
};
