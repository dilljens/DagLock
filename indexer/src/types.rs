#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique escrow identifier.
pub type EscrowId = String;

/// Unique offer identifier.
pub type OfferId = String;

/// Kaspa address string (e.g. "kaspa:qz2q...").
pub type Address = String;

/// Transaction ID (hex).
pub type TxId = String;

/// Escrow status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscrowStatus {
    /// Lock tx broadcast but not yet confirmed.
    PendingConfirmation,
    /// Lock tx confirmed, escrow active.
    Active,
    /// Escrow is under dispute.
    Disputed,
    /// Funds released to recipient (Path A).
    Settled,
    /// Funds returned to depositor (Path B — timeout).
    Refunded,
    /// Escrow cancelled before completion.
    Cancelled,
    /// Timeout reached without settlement.
    Expired,
}

impl EscrowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            EscrowStatus::PendingConfirmation => "pending_confirmation",
            EscrowStatus::Active => "active",
            EscrowStatus::Disputed => "disputed",
            EscrowStatus::Settled => "settled",
            EscrowStatus::Refunded => "refunded",
            EscrowStatus::Cancelled => "cancelled",
            EscrowStatus::Expired => "expired",
        }
    }

    pub fn parse_status(s: &str) -> Option<Self> {
        match s {
            "pending_confirmation" => Some(Self::PendingConfirmation),
            "active" => Some(Self::Active),
            "disputed" => Some(Self::Disputed),
            "settled" => Some(Self::Settled),
            "refunded" => Some(Self::Refunded),
            "cancelled" => Some(Self::Cancelled),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}

/// Core escrow record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub id: EscrowId,
    pub lock_tx_id: TxId,
    pub lock_tx_output_index: u32,
    pub status: EscrowStatus,
    pub asset_type: String,
    pub buyer_address: Address,
    pub seller_address: Option<Address>,
    pub amount_sompi: i64,
    pub fee_sompi: i64,
    pub template_hash: Vec<u8>,
    pub expiration_daa_score: Option<i64>,
    pub disputed_at: Option<i64>,
    pub dispute_reason: Option<String>,
    pub cancelled_at: Option<i64>,
    pub expired_at: Option<i64>,
    pub created_at: i64,
    pub settled_at: Option<i64>,
    pub refunded_at: Option<i64>,
    pub mediator_key: Option<String>,
    pub dispute_mode: Option<String>,
    pub dispute_outcome: Option<String>,
    pub dispute_resolved_at: Option<i64>,
    pub price_at_creation: Option<f64>,
    pub price_currency: Option<String>,
    pub trade_hash: Option<String>,
    pub price_lock_time: Option<i64>,
    pub price_at_settlement: Option<f64>,
    pub price_source: Option<String>,
    pub price_type: Option<String>,
    pub invoice_id: Option<String>,
    pub memo: Option<String>,
    pub auto_settle_timeout: Option<i64>,
    // ── Mediation columns ────────────────────────────────────────────
    pub mediation_status: Option<String>,
    pub mediation_buyer_claim: Option<String>,
    pub mediation_seller_claim: Option<String>,
    pub mediation_result: Option<String>,
    pub mediation_expires_at: Option<i64>,
    pub mediation_buyer_accepted: Option<bool>,
    pub mediation_seller_accepted: Option<bool>,
    /// Hex-encoded Ed25519 public key for client-side chat encryption.
    pub chat_pubkey_buyer: Option<String>,
    /// Hex-encoded Ed25519 public key for client-side chat encryption.
    pub chat_pubkey_seller: Option<String>,
}

/// Create escrow request (from POST endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEscrowRequest {
    pub lock_tx_id: TxId,
    pub lock_tx_output_index: u32,
    pub buyer_address: Address,
    pub seller_address: Option<Address>,
    pub amount_sompi: i64,
    pub expiration_daa_score: Option<i64>,
    pub treasury_address: Option<Address>,
    #[serde(default)]
    pub asset_type: Option<String>,
    #[serde(default)]
    pub template_hash: Option<Vec<u8>>,
    #[serde(default)]
    pub mediator_key: Option<String>,
    pub dispute_mode: Option<String>,
    pub price_at_creation: Option<f64>,
    pub price_currency: Option<String>,
    #[serde(default)]
    pub trade_hash: Option<String>,
    #[serde(default)]
    pub price_type: Option<String>,
    pub invoice_id: Option<String>,
    pub memo: Option<String>,
    #[serde(default)]
    pub auto_settle_timeout: Option<i64>,
    /// Creator's Ed25519 public key hex for client-side chat encryption.
    #[serde(default)]
    pub chat_pubkey: Option<String>,
}

/// App registered by an integrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub callback_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub created_at: i64,
    pub owner_address: Address,
    pub is_active: bool,
}

/// API key for an app (response only — key_hash never exposed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub app_id: String,
    pub label: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
    pub tier: String,
    pub webhooks_enabled: bool,
}

/// Register a new app request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAppRequest {
    pub name: String,
    pub callback_url: Option<String>,
    pub owner_address: Address,
}

/// Response when creating an app (includes the plaintext key — shown once only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAppResponse {
    pub app: App,
    pub api_key: String,
    pub warning: String,
}

/// Offer record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offer {
    pub id: OfferId,
    pub creator_address: Address,
    pub side: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub amount_sompi: i64,
    pub counterparty_address: Option<Address>,
    pub status: String,
    pub expires_at: Option<i64>,
    pub created_at: i64,
    pub price_type: String,
    pub price_offset: Option<f64>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub current_price: Option<f64>,
    pub price_currency: String,
    pub price_updated_at: Option<i64>,
    /// "user" or "bot". Auto-tagged from account_flags on creation.
    #[serde(default = "default_creator_type")]
    pub creator_type: String,
    /// Optional human-readable description of what's being traded.
    #[serde(default)]
    pub memo: Option<String>,
    /// Deal type: "goods", "otc", "service", "custom".
    #[serde(default = "default_deal_type")]
    pub deal_type: String,
}

fn default_creator_type() -> String {
    "user".to_string()
}

fn default_deal_type() -> String {
    "custom".to_string()
}

/// Create offer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOfferRequest {
    pub creator_address: Address,
    pub side: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub amount_sompi: i64,
    pub counterparty_address: Option<Address>,
    pub expires_at: Option<i64>,
    pub price_type: Option<String>,
    pub price_offset: Option<f64>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    /// "user" or "bot". Server auto-fills from account_flags if caller has a flag set.
    /// Bot scripts should explicitly set this to "bot" for honesty.
    #[serde(default)]
    pub creator_type: Option<String>,
    /// Optional human-readable description of what's being traded.
    #[serde(default)]
    pub memo: Option<String>,
    /// Deal type: "goods", "otc", "service", "custom".
    #[serde(default = "default_deal_type")]
    pub deal_type: String,
}

/// Accept offer request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptOfferRequest {
    pub counterparty_address: Address,
}

/// Reputation stats for an address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reputation {
    pub address: Address,
    pub trade_count: i64,
    pub recent_trade_count: i64,
    pub total_volume_sompi: i64,
    pub settled_count: i64,
    pub refunded_count: i64,
    pub disputed_count: i64,
    pub first_trade_at: Option<i64>,
    pub age_days: i64,
    pub dispute_rate: f64,
    pub refund_rate: f64,
    pub score: f64,
    pub telegram_handle: Option<String>,
    pub vouches_received: i64,
    pub vouches_given: i64,
    pub vouch_score: Option<f64>,
    pub mediator_stats: Option<MediatorStats>,
    pub trading_concentration: f64,
}

/// Create identity request (from POST endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityRequest {
    pub platform: String,
    pub handle: String,
    pub signed_message: String,
    pub signature_hex: String,
}

/// Receipt returned after settlement or refund.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: String,
    pub escrow_id: EscrowId,
    pub status: String,
    pub asset: String,
    pub amount_sompi: i64,
    pub fee_sompi: i64,
    pub buyer_address: Address,
    pub seller_address: Option<Address>,
    pub lock_tx_id: TxId,
    pub lock_tx_output_index: u32,
    pub expiration_daa_score: Option<i64>,
    pub disputed_at: Option<i64>,
    pub dispute_reason: Option<String>,
    pub cancelled_at: Option<i64>,
    pub expired_at: Option<i64>,
    pub settled_at: Option<i64>,
    pub refunded_at: Option<i64>,
    pub verification: ReceiptVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptVerification {
    pub covenant_verified: bool,
    pub signatures_verified: bool,
    pub fee_compliant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub network: String,
    pub daa_score: u64,
    pub block_count: u64,
    pub difficulty: f64,
    pub bps: f64,
    pub daglock_kas_template_hash: Option<String>,
    pub daglock_krc20_template_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimate {
    pub amount_kas: String,
    pub fee_kas: String,
    pub fee_percentage: f64,
    pub network_fee_estimate: String,
    pub miner_fee_budget: String,
}

// ── Multi-Party Escrow Types ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiEscrow {
    pub id: EscrowId,
    pub lock_tx_id: TxId,
    pub parties: Vec<String>,
    pub shares: Vec<i64>,
    pub total_amount: i64,
    pub status: String,
    pub created_at: i64,
    pub settled_at: Option<i64>,
    pub refunded_at: Option<i64>,
    pub signatures: Vec<String>,
}

/// Daily aggregated statistics record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStat {
    pub date: String,
    pub escrows_created: i64,
    pub escrows_settled: i64,
    pub volume_sompi: i64,
    pub fees_sompi: i64,
    pub active_escrows: i64,
    pub open_offers: i64,
    pub kas_usd_price: Option<f64>,
    pub total_users: i64,
}

/// Live aggregate summary of the entire system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSummary {
    pub total_escrows: i64,
    pub total_volume_sompi: i64,
    pub total_fees_sompi: i64,
    pub active_escrows: i64,
    pub total_users: i64,
    pub open_offers: i64,
    pub uptime_seconds: i64,
}

/// Stats response.
#[derive(Debug, Serialize, Deserialize)]
pub struct StatsResponse {
    pub total_escrows: i64,
    pub active_escrows: i64,
    pub disputed_escrows: i64,
    pub settled_escrows: i64,
    pub refunded_escrows: i64,
    pub cancelled_escrows: i64,
    pub total_volume_kas: String,
    pub total_fees_collected_kas: String,
    pub unique_buyers: i64,
    pub unique_sellers: i64,
}

/// Dispute evidence record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisputeEvidence {
    pub id: String,
    pub escrow_id: EscrowId,
    pub submitted_by: Address,
    pub content: String,
    pub content_hash: String,
    pub signed_message: Option<String>,
    pub created_at: i64,
}

/// Create evidence request (from POST endpoint).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvidenceRequest {
    pub content: String,
    pub signed_message: Option<String>,
}

/// Resolve dispute request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveDisputeRequest {
    pub outcome: String, // "expunge" or "uphold"
    pub resolved_by: Address,
}

/// Juror registration record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurorRegistration {
    pub address: Address,
    pub registered_at: i64,
    pub total_cases_assigned: i64,
    pub total_cases_voted: i64,
    pub reliability_score: f64,
}

/// Request to register as a juror.
/// Jury case record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuryCase {
    pub id: String,
    pub escrow_id: EscrowId,
    pub status: String,
    pub juror_count: i64,
    pub threshold: i64,
    pub votes_for_seller: i64,
    pub votes_for_buyer: i64,
    pub created_at: i64,
    pub decided_at: Option<i64>,
    pub outcome: Option<String>,
    pub jurors: Vec<String>,
    pub escalation_level: i64,
    pub escalation_deadline: Option<i64>,
    pub mediation_log: Option<String>,
    pub revealed_chat_key_enc: Option<String>,
    pub revealed_at: Option<i64>,
    pub evidence_cleared_at: Option<i64>,
}

/// Cast vote request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVoteRequest {
    pub vote: String,
    pub reasoning: Option<String>,
}

// ── AI Mediation Types ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediationOutcome {
    Refund,
    Payout,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationResult {
    pub outcome: MediationOutcome,
    pub buyer_share_basis: i64,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationRequest {
    pub buyer_claim: String,
    pub seller_claim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationResponse {
    pub case_id: String,
    pub recommendation: Option<MediationResult>,
    pub expires_at: i64,
    pub mediation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediationAccept {
    pub accept: bool,
}

/// Mediator stats for reputation display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediatorStats {
    pub disputes_mediated: i64,
    pub rulings_accepted: i64,
    pub acceptance_rate: f64,
    pub years_active: f64,
    pub score: f64,
}

/// Escrow message record.
/// Request to reveal chat key to jury.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevealChatKeyRequest {
    pub encrypted_chat_key: String,
}

/// Decrypted message evidence for jury.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMessage {
    pub id: String,
    pub sender_address: Address,
    pub decrypted_content: String,
    pub created_at: i64,
    pub anchor_tx_id: Option<String>,
    pub anchor_daa_score: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowMessage {
    pub id: String,
    pub escrow_id: EscrowId,
    pub sender_address: Address,
    #[deprecated(note = "Use content_enc from list response instead")]
    pub content: String,
    pub created_at: i64,
}

/// Send message request — client-side encrypted.
/// The server stores ciphertext only; it never sees plaintext.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    /// Hex-encoded ciphertext (client-side encrypted).
    pub content_enc: String,
    /// Hex-encoded 12-byte nonce (24 hex chars).
    pub nonce: String,
    /// Ed25519 signature hex (128 hex chars) over sha256(content_enc || nonce || escrow_id || seq).
    pub chat_sig: String,
}

/// Message list response.

/// Anchored message with on-chain proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredMessage {
    pub id: String,
    pub sender_address: Address,
    pub content_enc: String,
    pub nonce: String,
    pub created_at: i64,
    pub anchor_tx_id: Option<String>,
    pub anchor_daa_score: Option<i64>,
    pub anchor_batch_hash: Option<String>,
}

/// Anchor batch summary for an escrow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorBatch {
    pub batch_hash: String,
    pub anchor_tx_id: Option<String>,
    pub anchor_daa_score: Option<i64>,
    pub message_count: i64,
    pub from_time: i64,
    pub to_time: i64,
}

/// Anchor summary response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorSummary {
    pub escrow_id: String,
    pub batch_count: i64,
    pub batches: Vec<AnchorBatch>,
}

/// Typed error codes for structured API responses.
#[derive(Debug, Clone, Copy)]
pub enum ApiErrorCode {
    InternalError,
    InvalidAddress,
    InvalidAmount,
    InvalidTradeHash,
    InvalidTemplate,
    EscrowNotFound,
    EscrowNotActive,
    EscrowAlreadyFinalized,
    Unauthorized,
    Forbidden,
    DuplicateLock,
    SelfReferential,
    VerificationFailed,
    OfferNotFound,
    OfferNotAvailable,
    AppNotFound,
    KeyNotFound,
    WebhookNotFound,
    InvalidEvent,
    InsufficientJurors,
    PreimageMismatch,
    InvalidPreimage,
    DepositNotFound,
    DepositAlreadyFinalized,
    MediationInProgress,
    MediationNotFound,
    MediationAlreadyAccepted,
    MediationExpired,
}

impl ApiErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InternalError => "internal_error",
            Self::InvalidAddress => "invalid_address",
            Self::InvalidAmount => "invalid_amount",
            Self::InvalidTradeHash => "invalid_trade_hash",
            Self::InvalidTemplate => "invalid_template",
            Self::EscrowNotFound => "escrow_not_found",
            Self::EscrowNotActive => "escrow_not_active",
            Self::EscrowAlreadyFinalized => "escrow_already_finalized",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::DuplicateLock => "duplicate_lock",
            Self::SelfReferential => "self_referential",
            Self::VerificationFailed => "verification_failed",
            Self::OfferNotFound => "offer_not_found",
            Self::OfferNotAvailable => "offer_not_available",
            Self::AppNotFound => "app_not_found",
            Self::KeyNotFound => "key_not_found",
            Self::WebhookNotFound => "webhook_not_found",
            Self::InvalidEvent => "invalid_event",
            Self::InsufficientJurors => "insufficient_jurors",
            Self::PreimageMismatch => "preimage_mismatch",
            Self::InvalidPreimage => "invalid_preimage",
            Self::DepositNotFound => "deposit_not_found",
            Self::DepositAlreadyFinalized => "deposit_already_finalized",
            Self::MediationInProgress => "mediation_in_progress",
            Self::MediationNotFound => "mediation_not_found",
            Self::MediationAlreadyAccepted => "mediation_already_accepted",
            Self::MediationExpired => "mediation_expired",
        }
    }

    pub fn msg(self) -> &'static str {
        match self {
            Self::InternalError => "An internal error occurred.",
            Self::InvalidAddress => "Invalid Kaspa address format.",
            Self::InvalidAmount => "Amount must be positive and within limits.",
            Self::InvalidTradeHash => "Must be 64 hex characters (32 bytes).",
            Self::InvalidTemplate => "Not a known DagLock covenant template.",
            Self::EscrowNotFound => "No escrow found with this ID.",
            Self::EscrowNotActive => "Escrow is not in an active state.",
            Self::EscrowAlreadyFinalized => "Escrow is already finalized.",
            Self::Unauthorized => "Missing or invalid authentication.",
            Self::Forbidden => "Not authorized for this action.",
            Self::DuplicateLock => "An escrow already exists for this UTXO.",
            Self::SelfReferential => "Buyer and seller cannot be the same address.",
            Self::VerificationFailed => "On-chain verification failed.",
            Self::OfferNotFound => "Offer not found.",
            Self::OfferNotAvailable => "Offer is no longer available.",
            Self::AppNotFound => "App not found.",
            Self::KeyNotFound => "API key not found.",
            Self::WebhookNotFound => "Webhook not found.",
            Self::InvalidEvent => "Invalid event type.",
            Self::InsufficientJurors => "Not enough registered jurors.",
            Self::PreimageMismatch => "Preimage does not match trade hash.",
            Self::InvalidPreimage => "Preimage must be valid hex.",
            Self::DepositNotFound => "No deposit found for this escrow.",
            Self::DepositAlreadyFinalized => "Deposit is already finalized.",
            Self::MediationInProgress => "Mediation is already in progress for this escrow.",
            Self::MediationNotFound => "No mediation found for this escrow.",
            Self::MediationAlreadyAccepted => "You have already accepted the mediation outcome.",
            Self::MediationExpired => "Mediation has expired. Escalate to jury.",
        }
    }
}

/// Price alert record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: String,
    pub address: String,
    pub target_price: f64,
    pub direction: String,
    pub triggered: bool,
    pub created_at: i64,
    pub triggered_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    pub timestamp: i64,
    pub price_usd: f64,
}

/// Create price alert request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePriceAlertRequest {
    pub address: String,
    pub target_price: f64,
    pub direction: String,
}

/// API error response.
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: ApiErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetail {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ApiErrorDetail {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vouch {
    pub id: String,
    pub voucher_address: Address,
    pub subject_address: Address,
    pub escrow_id: Option<String>,
    pub note: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVouchRequest {
    pub subject_address: Address,
    pub escrow_id: Option<String>,
    pub note: Option<String>,
}

// ── Deposit (Security Deposit Covenant) Types ────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deposit {
    pub id: String,
    pub escrow_id: EscrowId,
    pub party1_address: Address,
    pub party2_address: Address,
    pub deposit_amount: i64,
    pub status: String,
    pub deposit_tx_id: Option<String>,
    pub timeout: i64,
    pub created_at: i64,
    pub released_at: Option<i64>,
    pub forfeited_at: Option<i64>,
    pub forfeited_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDepositRequest {
    pub party1_address: Address,
    pub party2_address: Address,
    pub deposit_amount: i64,
    pub deposit_tx_id: Option<String>,
    pub party1_pubkey: Option<String>,
    pub party2_pubkey: Option<String>,
    pub timeout: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseDepositRequest {
    pub party1_address: Address,
    pub party2_address: Address,
    pub party1_signature: String,
    pub party2_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForfeitDepositRequest {
    pub forfeited_to: String,
    pub jury_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepDepositsResponse {
    pub swept: Vec<String>,
    pub count: usize,
}

// ── Vault Types ─────────────────────────────────────────────────

/// Vault type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultType {
    Time,
    Beneficiary,
    Deadman,
    Inheritance,
    Multisig,
}

/// Vault status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VaultStatus {
    Locked,
    Unlocked,
    Expired,
    Transferred,
}

/// Vault record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: String,
    pub owner_address: Address,
    pub beneficiary_address: Option<Address>,
    pub vault_type: VaultType,
    pub status: VaultStatus,
    pub amount_sompi: i64,
    pub timeout: i64,
    pub lock_tx_id: Option<String>,
    pub lock_tx_output_index: Option<i64>,
    pub created_at: i64,
    pub unlocked_at: Option<i64>,
    pub expires_at: Option<i64>,
    /// Hex-encoded 32-byte owner public key (for indexer auto-sweep).
    pub owner_pubkey_hex: Option<String>,
    /// Transaction ID of the sweep broadcast (for idempotency).
    pub sweep_tx_id: Option<String>,
}

/// Create vault request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVaultRequest {
    pub owner_address: Address,
    pub beneficiary_address: Option<Address>,
    pub vault_type: VaultType,
    pub amount_sompi: i64,
    pub timeout: i64,
    pub lock_tx_id: Option<String>,
    pub lock_tx_output_index: Option<i64>,
    pub owner_pubkey_hex: Option<String>,
}

/// Vault list response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct VaultListResponse {
    pub vaults: Vec<Vault>,
    pub total: i64,
}

/// Withdraw vault request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawVaultRequest {
    pub owner_address: Address,
    pub signature: String,
}

/// Transfer vault request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TransferVaultRequest {
    pub beneficiary_address: Address,
    pub owner_address: Address,
    pub signature: String,
}

// ── Milestone Escrow Types ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneEscrow {
    pub id: EscrowId,
    pub lock_tx_id: TxId,
    pub buyer_address: Address,
    pub seller_address: Address,
    pub total_amount: i64,
    pub milestone_amounts: Vec<i64>,
    pub milestone_timeouts: Vec<i64>,
    pub current_milestone: i32,
    pub milestone_statuses: Vec<String>,
    pub status: String,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMilestoneRequest {
    pub lock_tx_id: TxId,
    pub buyer_address: Address,
    pub seller_address: Address,
    pub total_amount: i64,
    pub milestone_amounts: Vec<i64>,
    pub milestone_timeouts: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneListResponse {
    pub milestones: Vec<MilestoneEscrow>,
    pub total: i64,
}

// ── Invoices ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Settled,
    Disputed,
    Refunded,
    Cancelled,
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvoiceStatus::Draft => write!(f, "draft"),
            InvoiceStatus::Sent => write!(f, "sent"),
            InvoiceStatus::Paid => write!(f, "paid"),
            InvoiceStatus::Settled => write!(f, "settled"),
            InvoiceStatus::Disputed => write!(f, "disputed"),
            InvoiceStatus::Refunded => write!(f, "refunded"),
            InvoiceStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<&str> for InvoiceStatus {
    fn from(s: &str) -> Self {
        match s {
            "draft" => InvoiceStatus::Draft,
            "sent" => InvoiceStatus::Sent,
            "paid" => InvoiceStatus::Paid,
            "settled" => InvoiceStatus::Settled,
            "disputed" => InvoiceStatus::Disputed,
            "refunded" => InvoiceStatus::Refunded,
            "cancelled" => InvoiceStatus::Cancelled,
            _ => InvoiceStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub freelancer_address: Address,
    pub client_address: Option<Address>,
    pub escrow_id: Option<String>,
    pub description: String,
    pub amount_sompi: i64,
    pub due_date: Option<i64>,
    pub status: String,
    pub created_at: i64,
    pub paid_at: Option<i64>,
    pub settled_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub description: String,
    pub amount_sompi: i64,
    pub due_date: Option<i64>,
    pub client_email: Option<String>,
}

// ── Subscription Types ────────────────────────────────────────────

/// Recurring subscription payment record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub payer_address: Address,
    pub recipient_address: Address,
    pub total_amount: i64,
    pub installment_amount: i64,
    pub interval_seconds: i64,
    pub start_time: i64,
    pub current_period: i64,
    pub max_periods: i64,
    pub status: String,
    pub created_at: i64,
    pub cancelled_at: Option<i64>,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub id: Option<String>,
    pub payer_address: Address,
    pub recipient_address: Address,
    pub total_amount: i64,
    pub installment_amount: i64,
    pub interval_seconds: i64,
    pub start_time: i64,
    pub max_periods: i64,
    pub current_period: Option<i64>,
    pub lock_tx_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    pub subscriptions: Vec<Subscription>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawSubscriptionRequest {
    pub recipient_address: Address,
}

// ── Payment Session Types ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSession {
    pub id: String,
    pub app_id: String,
    pub escrow_id: Option<String>,
    pub amount_sompi: i64,
    pub asset_type: String,
    pub seller_address: Address,
    pub memo: Option<String>,
    pub status: String,
    pub buyer_address: Option<Address>,
    pub created_at: i64,
    pub expires_at: i64,
    pub webhook_url: Option<String>,
    pub redirect_url: Option<String>,
}

// ── Shared Helpers ────────────────────────────────────────────────

use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

/// Generate a short prefixed ID (e.g. "esc_abc123", "off_def456").
pub fn generate_id(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().to_string().replace('-', ""))
}

/// Standard internal server error response.
pub fn internal_error() -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!(ApiError::new(
            "internal_error",
            "An internal error occurred."
        ))),
    )
}

/// Standard not found error response.
pub fn not_found(entity: &str, id: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!(ApiError::new(
            "not_found",
            format!("No {} found with id '{}'", entity, id)
        ))),
    )
}

/// Standard bad request with custom code and message.
pub fn bad_request(code: &str, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard forbidden error.
pub fn forbidden(code: &str, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard conflict error.
pub fn conflict(code: &str, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard unauthorized error.

/// In-memory price cache with 5-minute TTL.
use std::sync::Mutex;

static PRICE_CACHE: once_cell::sync::Lazy<Mutex<Option<(f64, i64)>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

/// Fetch KAS/USD price from CoinGecko with 5s timeout.
/// Caches the result for 5 minutes (300 seconds).
/// Returns None if both API and cache fail.
pub async fn fetch_kas_usd_price() -> Option<f64> {
    // Check cache first
    if let Ok(cache) = PRICE_CACHE.lock() {
        if let Some((price, ts)) = *cache {
            let now = chrono::Utc::now().timestamp();
            if now - ts < 300 {
                return Some(price);
            }
        }
    }

    // Fetch from API
    use std::time::Duration;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("DagLock/0.1 (https://daglock.com; daglock@daglock.com)")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let resp = match client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Price oracle: HTTP request failed: {e}");
            return None;
        }
    };
    let price_json: serde_json::Value = match resp.json().await {
        Ok(j) => j,
        Err(e) => {
            tracing::warn!("Price oracle: JSON parse failed: {e}");
            return None;
        }
    };
    let price = match price_json["kaspa"]["usd"].as_f64() {
        Some(p) => p,
        None => {
            tracing::warn!("Price oracle: unexpected response format: {}", price_json);
            return None;
        }
    };

    if price > 0.0 {
        // Update cache
        if let Ok(mut cache) = PRICE_CACHE.lock() {
            *cache = Some((price, chrono::Utc::now().timestamp()));
        }
        Some(price)
    } else {
        None
    }
}

/// Account flags — per-address metadata (is_bot, label, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountFlags {
    pub address: Address,
    pub is_bot: bool,
    pub label: Option<String>,
    pub updated_at: i64,
}

/// Request to set account flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAccountFlagsRequest {
    pub address: Address,
    pub is_bot: bool,
    pub label: Option<String>,
}

pub fn unauthorized(message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!(ApiError::new("unauthorized", message))),
    )
}
