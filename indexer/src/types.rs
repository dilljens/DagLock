use serde::{Deserialize, Serialize};

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
    pub dispute_outcome: Option<String>,
    pub dispute_resolved_at: Option<i64>,
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
}

/// Verified social identity linked to a Kaspa address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedIdentity {
    pub address: Address,
    pub platform: String,
    pub handle: String,
    pub verified_at: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VouchListResponse {
    pub vouches: Vec<Vouch>,
    pub total: i64,
}
