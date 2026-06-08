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

/// Verified social identity linked to a Kaspa address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JuryRegisterRequest {
    pub address: Address,
}

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
}

/// Jury vote record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JuryVote {
    pub case_id: String,
    pub juror_address: Address,
    pub vote: String,
    pub voted_at: i64,
    pub reasoning: Option<String>,
}

/// Cast vote request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastVoteRequest {
    pub vote: String,
    pub reasoning: Option<String>,
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

/// Escrow message record (encrypted at rest).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowMessage {
    pub id: String,
    pub escrow_id: EscrowId,
    pub sender_address: Address,
    pub content: String, // decrypted plaintext
    pub created_at: i64,
}

/// Send message request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// Message list response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct MessageListResponse {
    pub messages: Vec<EscrowMessage>,
    pub total: i64,
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
#[allow(dead_code)]
pub struct VouchListResponse {
    pub vouches: Vec<Vouch>,
    pub total: i64,
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


// ── Shared Helpers ────────────────────────────────────────────────

use axum::http::StatusCode;
use axum::Json;
use serde_json::{json, Value};

/// Generate a short prefixed ID (e.g. "esc_abc123", "off_def456").
pub fn generate_id(prefix: &str) -> String {
    format!(
        "{}_{}",
        prefix,
        Uuid::new_v4().to_string().split('-').next().unwrap()
    )
}

/// Standard internal server error response.
pub fn internal_error() -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!(ApiError::new("internal_error", "An internal error occurred."))),
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
pub fn bad_request(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard forbidden error.
pub fn forbidden(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard conflict error.
pub fn conflict(code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::CONFLICT,
        Json(json!(ApiError::new(code, message))),
    )
}

/// Standard unauthorized error.


/// Fetch KAS/USD price from CoinGecko with 5s timeout.
/// Returns None if the request fails or price is zero.
pub async fn fetch_kas_usd_price() -> Option<f64> {
    use std::time::Duration;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=kaspa&vs_currencies=usd")
        .send()
        .await
        .ok()?;
    let price_json: serde_json::Value = resp.json().await.ok()?;
    let price = price_json["kaspa"]["usd"].as_f64()?;
    if price > 0.0 { Some(price) } else { None }
}

pub fn unauthorized(message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!(ApiError::new("unauthorized", message))),
    )
}
