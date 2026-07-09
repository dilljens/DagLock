const API_BASE = import.meta.env.VITE_API_URL || "";
if (import.meta.env.DEV) {
	console.log("[DagLock] API_BASE:", API_BASE);
}

// Request timeout (30 seconds)
const REQUEST_TIMEOUT_MS = 30_000;

async function fetchWithTimeout(url: string, init?: RequestInit): Promise<Response> {
	const controller = new AbortController();
	const timeoutId = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
	try {
		const response = await fetch(url, { ...init, signal: controller.signal });
		return response;
	} finally {
		clearTimeout(timeoutId);
	}
}

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

export type PriceAlert = {
	id: string;
	address: string;
	target_price: number;
	direction: string;
	triggered: boolean;
	created_at: number;
	triggered_at: number | null;
};

export type PriceHistoryPoint = {
	timestamp: number;
	price_usd: number;
};

export type DailyStat = {
	date: string;
	escrows_created: number;
	escrows_settled: number;
	volume_sompi: number;
	fees_sompi: number;
	active_escrows: number;
	open_offers: number;
	kas_usd_price: number | null;
	total_users: number;
};

export type AccountFlags = {
	address: string;
	is_bot: boolean;
	label?: string | null;
	updated_at: number;
};

export type LiveSummary = {
	total_escrows: number;
	total_volume_sompi: number;
	total_fees_sompi: number;
	active_escrows: number;
	total_users: number;
	open_offers: number;
	uptime_seconds: number;
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
	creator_type?: string;
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
	memo?: string | null;
	auto_settle_timeout?: number | null;
	chat_pubkey_buyer?: string | null;
	chat_pubkey_seller?: string | null;
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
	invoice_id?: string;
	memo?: string;
	auto_settle_timeout?: number;
	chat_pubkey?: string;
};

export type AuthHeaders = {
	address: string;
	signature: string;
	message: string;
};

export type InvoiceData = {
	id: string;
	freelancer_address: string;
	client_address: string | null;
	escrow_id: string | null;
	description: string;
	amount_sompi: number;
	due_date: number | null;
	status: string;
	created_at: number;
	paid_at: number | null;
	settled_at: number | null;
};

export type InvoiceResponse = {
	invoice: InvoiceData;
	escrow_status: string | null;
	link: string;
};

export type ApiKey = {
	key_id: string;
	app_id: string;
	label: string;
	created_at: number;
	last_used_at: number | null;
	is_active: boolean;
	tier: string;
	webhooks_enabled: boolean;
};

export type App = {
	id: string;
	name: string;
	callback_url: string | null;
	webhook_secret: string | null;
	created_at: number;
	owner_address: string;
	is_active: boolean;
};

export type CreateInvoiceRequest = {
	description: string;
	amount_sompi: number;
	due_date?: number;
	client_email?: string;
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
	escalation_level: number;
	escalation_deadline?: number | null;
	mediation_log?: string | null;
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
	content?: string;
	content_enc?: string;
	nonce?: string;
	chat_sig?: string;
	seq?: number;
	created_at: number;
	anchor_tx_id?: string | null;
	anchor_daa_score?: number | null;
	anchor_batch_hash?: string | null;
};

export type AnchorBatch = {
	batch_hash: string;
	anchor_tx_id: string | null;
	anchor_daa_score: number | null;
	message_count: number;
	from_time: number;
	to_time: number;
};

export type AnchorSummary = {
	escrow_id: string;
	batch_count: number;
	batches: AnchorBatch[];
};

export type SendMessageRequest = {
	content?: string;
	content_enc?: string;
	nonce?: string;
	chat_sig?: string;
};

// ── Deposit Types ──────────────────────────────────────────────

export type Deposit = {
	id: string;
	escrow_id: string;
	party1_address: string;
	party2_address: string;
	deposit_amount: number;
	status: string;
	deposit_tx_id?: string | null;
	timeout: number;
	created_at: number;
	released_at?: number | null;
	forfeited_at?: number | null;
	forfeited_to?: string | null;
};

export type CreateDepositRequest = {
	party1_address: string;
	party2_address: string;
	deposit_amount: number;
	deposit_tx_id?: string;
	party1_pubkey?: string;
	party2_pubkey?: string;
	timeout?: number;
};

export type ReleaseDepositRequest = {
	party1_address: string;
	party2_address: string;
	party1_signature: string;
	party2_signature: string;
};

export type ForfeitDepositRequest = {
	forfeited_to: string;
	jury_signature: string;
};

// ── Multi-Party Escrow Types ────────────────────────────────────

export type MultiEscrow = {
	id: string;
	lock_tx_id: string;
	parties: string[];
	shares: number[];
	total_amount: number;
	status: string;
	created_at: number;
	settled_at: number | null;
	refunded_at: number | null;
	signatures: string[];
};

export type CreateMultiRequest = {
	lock_tx_id: string;
	parties: string[];
	shares: number[];
	total_amount: number;
};

// ── Milestone Types ──────────────────────────────────────────────

export type MilestoneEscrow = {
	id: string;
	lock_tx_id: string;
	buyer_address: string;
	seller_address: string;
	total_amount: number;
	milestone_amounts: number[];
	milestone_timeouts: number[];
	current_milestone: number;
	milestone_statuses: string[];
	status: string;
	created_at: number;
	completed_at: number | null;
};

export type CreateMilestoneRequest = {
	lock_tx_id: string;
	buyer_address: string;
	seller_address: string;
	total_amount: number;
	milestone_amounts: number[];
	milestone_timeouts: number[];
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

// ── Mediation Types ───────────────────────────────────────────────

export type MediationResult = {
	outcome: "refund" | "payout" | "split";
	buyer_share_basis: number;
	reasoning: string;
};

export type MediationStatus = {
	escrow_id: string;
	mediation_status: string;
	recommendation: MediationResult | null;
	expires_at: number | null;
	buyer_accepted: boolean;
	seller_accepted: boolean;
	both_accepted: boolean;
};

export type MediationRequest = {
	buyer_claim: string;
	seller_claim: string;
};

export type MediationResponse = {
	case_id: string;
	recommendation: MediationResult | null;
	expires_at: number;
	mediation_status: string;
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

export type EvidenceMessage = {
	id: string;
	sender_address: string;
	decrypted_content: string;
	created_at: number;
	anchor_tx_id?: string | null;
	anchor_daa_score?: number | null;
};

export type RevealChatKeyRequest = {
	encrypted_chat_key: string;
};

// ── Vault Types ─────────────────────────────────────────────────

export type VaultType = "time" | "beneficiary" | "deadman" | "inheritance" | "multisig";

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

// ── Subscription Types ───────────────────────────────────────────

export type Subscription = {
	id: string;
	payer_address: string;
	recipient_address: string;
	total_amount: number;
	installment_amount: number;
	interval_seconds: number;
	current_period: number;
	max_periods: number;
	status: string;
	created_at: number;
	cancelled_at?: number | null;
	completed_at?: number | null;
};

export type CreateSubscriptionRequest = {
	payer_address: string;
	recipient_address: string;
	total_amount: number;
	installment_amount: number;
	interval_seconds: number;
	max_periods: number;
	start_time: number;
	lock_tx_id?: string;
};

async function loadAuthJson<T>(path: string, auth: AuthHeaders): Promise<T> {
	const response = await fetchWithTimeout(API_BASE + path, {
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

/** Extract a human-readable error from an API error response. */
async function apiError(response: Response): Promise<string> {
	if (response.status === 429) {
		return "Too many requests. Please wait a moment and try again.";
	}
	try {
		const json = await response.json();
		return json.message || json.error || response.statusText;
	} catch {
		return response.statusText;
	}
}

async function loadJson<T>(path: string): Promise<T> {
	const response = await fetchWithTimeout(API_BASE + path);
	if (!response.ok) {
		const msg = await apiError(response);
		throw new Error(msg);
	}
	return response.json() as Promise<T>;
}

async function postJson<T>(path: string, body: unknown, auth?: AuthHeaders): Promise<T> {
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
	};
	if (auth) {
		headers["X-Daglock-Address"] = auth.address;
		headers["X-Daglock-Signature"] = auth.signature;
		headers["X-Daglock-Message"] = auth.message;
	}
	const response = await fetchWithTimeout(API_BASE + path, {
		method: "POST",
		headers,
		body: JSON.stringify(body),
	});
	if (!response.ok) {
		const msg = await apiError(response);
		throw new Error(msg);
	}
	return response.json() as Promise<T>;
}

/** DELETE with auth headers. */
async function deleteJson<T>(path: string, auth?: AuthHeaders): Promise<T> {
	const headers: Record<string, string> = {};
	if (auth) {
		headers["X-Daglock-Address"] = auth.address;
		headers["X-Daglock-Signature"] = auth.signature;
		headers["X-Daglock-Message"] = auth.message;
	}
	const response = await fetchWithTimeout(API_BASE + path, { method: "DELETE", headers });
	if (!response.ok) {
		const msg = await apiError(response);
		throw new Error(msg);
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
	const response = await fetchWithTimeout(API_BASE + path, {
		method: "POST",
		headers,
	});
	if (!response.ok) {
		const text = await response.text();
		throw new Error(text);
	}
	return response.json() as Promise<T>;
}

export type CompileResponse = {
	script: string;
	template_hash: string;
	abi: { name: string }[];
};

// KRC-20 token types
export type TokenSummary = {
	ticker: string;
	price_kas: number | null;
	volume_24h_sompi: number;
	trades_24h: number;
	total_trades: number;
	active_offers: number;
	last_trade_at: number | null;
};

export type TokenTrade = {
	escrow_id: string;
	amount_sompi: number;
	status: string;
	created_at: number;
	buyer_address: string;
	seller_address?: string | null;
};

export type TokenDetail = TokenSummary & {
	trades: TokenTrade[];
};

export type TokenChartPoint = {
	timestamp: number;
	volume_kas: number;
};

export type TokenRegistryEntry = {
	id: string;
	ticker: string;
	name: string;
	total_supply: number;
	decimals: number;
	mint_mode: string;
	owner_address?: string | null;
	covenant_address?: string | null;
	template_hash?: string | null;
	status: string;
	created_at: number;
};

export type DeployTokenRequest = {
	name: string;
	ticker: string;
	total_supply: number;
	decimals: number;
	mint_mode: string;
	owner_address?: string;
};

export type UpdateTokenRequest = {
	status?: string;
	covenant_address?: string;
	deploy_tx_id?: string;
};

export const api = {
	health: () => loadJson<Health>("/v1/health"),
	compile: (template: string, params: Record<string, string>) =>
		postJson<CompileResponse>("/v1/compile", { template, params }),
	stats: () => loadJson<Stats>("/v1/stats"),

	// Vaults
	vaults: (owner?: string) =>
		loadJson<{ vaults: Vault[]; total: number }>(
			`/v1/vaults${owner ? `?owner=${encodeURIComponent(owner)}` : ""}`,
		),
	vault: (id: string) => loadJson<Vault>(`/v1/vaults/${encodeURIComponent(id)}`),
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
	escrow: (id: string) => loadJson<Escrow>(`/v1/escrows/${encodeURIComponent(id)}`),
	createEscrow: (req: CreateEscrowRequest) => postJson<Escrow>("/v1/escrows", req),
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
	autoSettleEscrow: (id: string) =>
		postEmpty<{ status: string; escrow_id: string; method: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/auto-settle`,
		),
	swapEscrow: (id: string, preimage: string) =>
		postJson<{ status: string; escrow_id: string; method: string; preimage_hash: string }>(
			`/v1/escrows/${encodeURIComponent(id)}/swap`,
			{ preimage },
		),
	generateSwap: () => loadJson<{ secret: string; hash: string }>("/v1/swap/generate"),

	// Deposits (security deposit covenant)
	createDeposit: (escrowId: string, req: CreateDepositRequest) =>
		postJson<Deposit>(`/v1/escrows/${encodeURIComponent(escrowId)}/deposit`, req),
	getDeposit: (escrowId: string) =>
		loadJson<Deposit>(`/v1/escrows/${encodeURIComponent(escrowId)}/deposit`),
	releaseDeposit: (escrowId: string, req: ReleaseDepositRequest) =>
		postJson<{ status: string; deposit_id: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/deposit/release`,
			req,
		),
	forfeitDeposit: (escrowId: string, req: ForfeitDepositRequest) =>
		postJson<{ status: string; deposit_id: string; escrow_id: string; forfeited_to: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/deposit/forfeit`,
			req,
		),
	sweepDeposits: () =>
		postJson<{ swept: string[]; count: number; total_stale: number }>("/v1/deposits/sweep", {}),

	// Invoices
	getInvoice: (id: string) => loadJson<InvoiceResponse>(`/v1/invoices/${encodeURIComponent(id)}`),
	createInvoice: (req: CreateInvoiceRequest, auth: AuthHeaders) =>
		postJson<{ id: string; link: string; invoice: InvoiceData }>("/v1/invoices", req, auth),
	listInvoices: (address: string, auth: AuthHeaders) =>
		loadAuthJson<{ invoices: InvoiceData[]; total: number }>(
			`/v1/invoices?address=${encodeURIComponent(address)}`,
			auth,
		),

	// Offers
	offers: (creator?: string) =>
		loadJson<{ offers: Offer[]; total: number }>(
			`/v1/offers?status=proposed${creator ? `&creator=${encodeURIComponent(creator)}` : ""}`,
		),
	createOffer: (req: CreateOfferRequest) => postJson<Offer>("/v1/offers", req),
	acceptOffer: (id: string, counterparty_address: string) =>
		postJson<{ status: string; offer_id: string }>(`/v1/offers/${encodeURIComponent(id)}/accept`, {
			counterparty_address,
		}),
	cancelOffer: (id: string) =>
		postEmpty<{ status: string; offer_id: string }>(`/v1/offers/${encodeURIComponent(id)}/cancel`),

	// Counter-offers
	counterOffer: (offerId: string, req: { amount_sompi?: number; message?: string }) =>
		postJson<{ status: string; id: string; offer_id: string }>(
			`/v1/offers/${encodeURIComponent(offerId)}/counter`,
			req,
		),
	listCounters: (offerId: string) =>
		loadJson<{ counters: any[]; total: number }>(
			`/v1/offers/${encodeURIComponent(offerId)}/counters`,
		),
	acceptCounter: (counterId: string) =>
		postEmpty<{ status: string; counter_id: string; offer_id: string }>(
			`/v1/counteroffers/${encodeURIComponent(counterId)}/accept`,
		),
	declineCounter: (counterId: string) =>
		postEmpty<{ status: string; counter_id: string }>(
			`/v1/counteroffers/${encodeURIComponent(counterId)}/decline`,
		),

	// Vouching
	vouch: (subjectAddress: string, auth: AuthHeaders, escrowId?: string, note?: string) =>
		postJson<{ status: string; vouch: Vouch }>(
			"/v1/vouches",
			{ subject_address: subjectAddress, escrow_id: escrowId, note },
			auth,
		),

	// Jury
	juryRegister: (auth: AuthHeaders) =>
		postJson<{ status: string; address: string }>("/v1/jury/register", {}, auth),
	juryUnregister: (auth: AuthHeaders) =>
		postJson<{ status: string; address: string }>("/v1/jury/unregister", {}, auth),
	juryCases: (auth: AuthHeaders) =>
		loadAuthJson<{ cases: JuryCase[]; total: number }>("/v1/jury/cases", auth),
	juryVote: (caseId: string, vote: string, reasoning?: string, auth?: AuthHeaders) =>
		postJson<{ status: string; vote: string; verdict?: string | null }>(
			`/v1/jury/cases/${encodeURIComponent(caseId)}/vote`,
			{ vote, reasoning },
			auth,
		),
	juryCandidates: () =>
		loadJson<{ candidates: JurorRegistration[]; total: number }>("/v1/jury/candidates"),

	// Messages
	sendMessage: (escrowId: string, data: SendMessageRequest, auth: AuthHeaders) =>
		postJson<{ status: string; message: EscrowMessage }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/messages`,
			data,
			auth,
		),
	submitChatPubkey: (escrowId: string, chatPubkey: string, auth: AuthHeaders) =>
		postJson<{ status: string; escrow_id: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/chat-pubkey`,
			{ chat_pubkey: chatPubkey },
			auth,
		),
	listMessages: (escrowId: string, auth: AuthHeaders) =>
		loadAuthJson<{
			messages: EscrowMessage[];
			total: number;
			escrow_id: string;
		}>(`/v1/escrows/${encodeURIComponent(escrowId)}/messages`, auth),
	getMessageAnchors: (escrowId: string, auth: AuthHeaders) =>
		loadAuthJson<AnchorSummary>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/messages/anchors`,
			auth,
		),

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
	receipt: (id: string) => loadJson<Receipt>(`/v1/receipts/${encodeURIComponent(id)}`),

	// Account flags
	getFlags: (address: string) =>
		loadJson<AccountFlags>(`/v1/flags/${encodeURIComponent(address)}`),
	setFlags: (req: { address: string; is_bot: boolean; label?: string | null }) =>
		postJson<{ status: string; address: string; is_bot: boolean }>("/v1/flags", req),

	// Email notifications
	getNotifications: (auth: AuthHeaders) => loadAuthJson<any>("/v1/notifications", auth),
	subscribeNotifications: (req: { email: string }, auth: AuthHeaders) =>
		postJson<any>("/v1/notifications", req, auth),
	verifyNotifications: (req: { code: string }, auth: AuthHeaders) =>
		postJson<any>("/v1/notifications/verify", req, auth),
	updateNotificationPrefs: (req: any, auth: AuthHeaders) =>
		postJson<any>("/v1/notifications/preferences", req, auth),

	// Milestones
	milestones: (address: string) =>
		loadJson<{ milestones: MilestoneEscrow[]; total: number }>(
			`/v1/milestones?address=${encodeURIComponent(address)}`,
		),
	milestone: (id: string) => loadJson<MilestoneEscrow>(`/v1/milestones/${encodeURIComponent(id)}`),
	createMilestone: (req: CreateMilestoneRequest) =>
		postJson<MilestoneEscrow>("/v1/milestones", req),
	releaseMilestone: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/milestones/${encodeURIComponent(id)}/release`,
		),
	approveMilestone: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/milestones/${encodeURIComponent(id)}/approve`,
		),
	disputeMilestone: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/milestones/${encodeURIComponent(id)}/dispute`,
		),
	refundMilestone: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/milestones/${encodeURIComponent(id)}/refund`,
		),
	completeMilestone: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/milestones/${encodeURIComponent(id)}/complete`,
		),

	// Multi-party escrows
	multiEscrows: (address: string) =>
		loadJson<{ multi_escrows: MultiEscrow[]; total: number }>(
			`/v1/multi-escrows?address=${encodeURIComponent(address)}`,
		),
	multiEscrow: (id: string) => loadJson<MultiEscrow>(`/v1/multi-escrows/${encodeURIComponent(id)}`),
	createMultiEscrow: (req: CreateMultiRequest) => postJson<MultiEscrow>("/v1/multi-escrows", req),
	signMultiEscrow: (id: string, address: string) =>
		postJson<{
			status: string;
			escrow_id: string;
			signature_count: number;
			parties_count: number;
			all_signed: boolean;
		}>(`/v1/multi-escrows/${encodeURIComponent(id)}/sign`, { address }),
	refundMultiEscrow: (id: string) =>
		postEmpty<{ status: string; escrow_id: string }>(
			`/v1/multi-escrows/${encodeURIComponent(id)}/refund`,
		),
	swapMultiEscrow: (id: string) =>
		postEmpty<{ status: string; escrow_id: string; method: string }>(
			`/v1/multi-escrows/${encodeURIComponent(id)}/swap`,
		),

	// API key management
	upgradeKeyTier: (appId: string, keyId: string, tier: string, adminToken: string) =>
		fetchWithTimeout(
			`${API_BASE}/v1/apps/${encodeURIComponent(appId)}/keys/${encodeURIComponent(keyId)}/tier`,
			{
				method: "PATCH",
				headers: {
					"Content-Type": "application/json",
					"X-Daglock-Admin": adminToken,
				},
				body: JSON.stringify({ tier }),
			},
		).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<{ status: string; key_id: string; app_id: string; tier: string }>;
		}),

	registerApp: (req: { name: string; owner_address: string; callback_url?: string }) =>
		postJson<{ app: App; api_key: string; warning: string }>("/v1/apps/register", req),

	getApp: (appId: string, apiKey: string) =>
		fetchWithTimeout(`${API_BASE}/v1/apps/${encodeURIComponent(appId)}`, {
			headers: { "X-Daglock-Api-Key": apiKey },
		}).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<App>;
		}),

	listApiKeys: (appId: string, apiKey: string) =>
		fetchWithTimeout(`${API_BASE}/v1/apps/${encodeURIComponent(appId)}/keys`, {
			headers: { "X-Daglock-Api-Key": apiKey },
		}).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<{ keys: ApiKey[]; total: number }>;
		}),

	createApiKey: (appId: string, apiKey: string) =>
		fetchWithTimeout(`${API_BASE}/v1/apps/${encodeURIComponent(appId)}/keys`, {
			method: "POST",
			headers: { "X-Daglock-Api-Key": apiKey },
		}).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<{ api_key: string; app_id: string; warning: string }>;
		}),

	deleteApiKey: (appId: string, keyId: string, apiKey: string) =>
		fetchWithTimeout(
			`${API_BASE}/v1/apps/${encodeURIComponent(appId)}/keys/${encodeURIComponent(keyId)}`,
			{
				method: "DELETE",
				headers: { "X-Daglock-Api-Key": apiKey },
			},
		).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<{ status: string }>;
		}),

	// Stats / analytics
	getDailyStats: (days?: number) =>
		loadJson<{ stats: DailyStat[]; days: number }>(`/v1/stats/daily${days ? `?days=${days}` : ""}`),
	getLiveSummary: () => loadJson<LiveSummary>("/v1/stats/summary"),

	// Price alerts
	createPriceAlert: (req: { address: string; target_price: number; direction: string }) =>
		postJson<PriceAlert>("/v1/price-alerts", req),
	listPriceAlerts: (address: string) =>
		loadJson<{ alerts: PriceAlert[]; total: number }>(
			`/v1/price-alerts?address=${encodeURIComponent(address)}`,
		),
	deletePriceAlert: (id: string) =>
		fetchWithTimeout(`${API_BASE}/v1/price-alerts/${encodeURIComponent(id)}`, {
			method: "DELETE",
		}).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json() as Promise<{ status: string; alert_id: string }>;
		}),

	// Network / prices
	networkPrice: () => loadJson<{ kas_usd: number; updated_at: number }>("/v1/network/price"),
	getPriceHistory: (days?: number) =>
		loadJson<{ points: PriceHistoryPoint[]; days: number }>(
			`/v1/network/price/history${days ? `?days=${days}` : ""}`,
		),
	explorer: () => loadJson<{ base_url: string }>("/v1/network/explorer"),

	// KRC-20 tokens
	tokens: () =>
		loadJson<{ tokens: TokenSummary[]; total: number }>("/v1/tokens"),
	token: (ticker: string) =>
		loadJson<TokenDetail>(`/v1/tokens/${encodeURIComponent(ticker)}`),
	tokenChart: (ticker: string, period?: string) =>
		loadJson<{ points: TokenChartPoint[] }>(
			`/v1/tokens/${encodeURIComponent(ticker)}/chart${period ? `?period=${period}` : ""}`,
		),
	registeredTokens: () =>
		loadJson<{ tokens: TokenRegistryEntry[]; total: number }>("/v1/tokens/registered"),
	deployToken: (req: DeployTokenRequest, auth: AuthHeaders) =>
		postJson<{ status: string; ticker: string }>("/v1/tokens/deploy", req, auth),
	updateToken: (ticker: string, req: UpdateTokenRequest, auth: AuthHeaders) =>
		fetchWithTimeout(`${API_BASE}/v1/tokens/${encodeURIComponent(ticker)}`, {
			method: "PATCH",
			headers: {
				"Content-Type": "application/json",
				"X-Daglock-Address": auth.address,
				"X-Daglock-Signature": auth.signature,
				"X-Daglock-Message": auth.message,
			},
			body: JSON.stringify(req),
		}).then((r) => {
			if (!r.ok) throw new Error(r.statusText);
			return r.json();
		}),

	// AI Mediation
	mediateEscrow: (id: string, body: MediationRequest, auth: AuthHeaders) =>
		postJson<MediationResponse>(`/v1/escrows/${encodeURIComponent(id)}/mediate`, body, auth),
	acceptMediation: (id: string, party: string, accept: boolean, auth: AuthHeaders) =>
		postJson<{
			status: string;
			escrow_id: string;
			outcome_executed?: boolean;
			waiting_for_other?: boolean;
		}>(`/v1/escrows/${encodeURIComponent(id)}/mediate/${party}/accept`, { accept }, auth),
	getMediation: (id: string) =>
		loadJson<MediationStatus>(`/v1/escrows/${encodeURIComponent(id)}/mediate`),

	// Evidence
	submitEvidence: (escrowId: string, content: string, signedMessage?: string, auth?: AuthHeaders) =>
		postJson<DisputeEvidence>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/evidence`,
			{ content, signed_message: signedMessage },
			auth,
		),
	resolveDispute: (escrowId: string, outcome: string, resolved_by: string, auth?: AuthHeaders) =>
		postJson<{ status: string; escrow_id: string; outcome: string }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/resolve-dispute`,
			{ outcome, resolved_by },
			auth,
		),
	juryActiveCases: (address: string) =>
		loadJson<{ count: number; cases: JuryCase[] }>(
			`/v1/jury/cases/active/${encodeURIComponent(address)}`,
		),

	// Chat reveal / evidence
	revealChatKey: (escrowId: string, encryptedKey: string, auth?: AuthHeaders) =>
		postJson<{ status: string; evidence_count: number }>(
			`/v1/escrows/${encodeURIComponent(escrowId)}/messages/reveal`,
			{ encrypted_chat_key: encryptedKey },
			auth,
		),
	getEvidence: (caseId: string, auth: AuthHeaders) =>
		loadAuthJson<{
			evidence: EvidenceMessage[];
			chat_pubkey_buyer: string | null;
			chat_pubkey_seller: string | null;
		}>(`/v1/jury/cases/${encodeURIComponent(caseId)}/evidence`, auth),
	clearEvidence: (caseId: string, auth?: AuthHeaders) =>
		postEmpty<{ status: string }>(
			`/v1/jury/cases/${encodeURIComponent(caseId)}/evidence/clear`,
			auth,
		),

	// Subscriptions
	subscriptions: (address?: string) =>
		loadJson<{ subscriptions: Subscription[]; total: number }>(
			`/v1/subscriptions${address ? `?address=${encodeURIComponent(address)}` : ""}`,
		),
	createSubscription: (req: CreateSubscriptionRequest, auth: AuthHeaders) =>
		postJson<Subscription>("/v1/subscriptions", req, auth),
	getSubscription: (id: string) =>
		loadJson<Subscription>(`/v1/subscriptions/${encodeURIComponent(id)}`),
	cancelSubscription: (id: string) =>
		postEmpty<{ status: string; subscription_id: string }>(
			`/v1/subscriptions/${encodeURIComponent(id)}/cancel`,
		),
	drawSubscription: (id: string) =>
		postEmpty<{ status: string; subscription_id: string; current_period: number; max_periods: number }>(
			`/v1/subscriptions/${encodeURIComponent(id)}/draw`,
		),
};
