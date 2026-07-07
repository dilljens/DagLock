//! Escrow business logic — separated from HTTP handlers.
//!
//! `EscrowService` owns all escrow lifecycle operations: create, settle,
//! refund, dispute, cancel, and atomic swap. Handlers delegate to this
//! layer for validation, auth checks, DB updates, and side effects.

use crate::auth::{
    parse_message, verify_cancel_authorization, verify_nonce, verify_refund_authorization,
    verify_settle_authorization, AuthContext, SignatureVerifier,
};
use crate::db::queries;
use crate::services::webhooks::{self, WebhookEvent};
use crate::types::*;
use crate::verification::verify_escrow_active;
use crate::websocket::WsEvent;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;

/// Error type returned by service methods.
/// Maps directly to HTTP status codes in handlers.
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    InvalidInput(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
    VerificationFailed(String),
    Internal(String),
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "{msg}"),
            Self::InvalidInput(msg) => write!(f, "{msg}"),
            Self::Unauthorized(msg) => write!(f, "{msg}"),
            Self::Forbidden(msg) => write!(f, "{msg}"),
            Self::Conflict(msg) => write!(f, "{msg}"),
            Self::VerificationFailed(msg) => write!(f, "{msg}"),
            Self::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl ServiceError {
    pub fn error_code(&self) -> &str {
        match self {
            Self::NotFound(_) => "escrow_not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "escrow_already_finalized",
            Self::VerificationFailed(_) => "verification_failed",
            Self::Internal(_) => "internal_error",
        }
    }
}

/// Escrow lifecycle service.
pub struct EscrowService<'a> {
    db: Pool<Sqlite>,
    ws_tx: &'a Sender<WsEvent>,
    sig_verifier: Arc<dyn SignatureVerifier>,
    verifier: Arc<dyn crate::verification::EscrowVerifier>,
    email_service: Option<std::sync::Arc<crate::services::email::EmailService>>,
}

#[allow(dead_code)]
impl<'a> EscrowService<'a> {
    pub fn new(
        db: Pool<Sqlite>,
        ws_tx: &'a Sender<WsEvent>,
        sig_verifier: Arc<dyn SignatureVerifier>,
        verifier: Arc<dyn crate::verification::EscrowVerifier>,
        email_service: Option<std::sync::Arc<crate::services::email::EmailService>>,
    ) -> Self {
        Self {
            db,
            ws_tx,
            sig_verifier,
            verifier,
            email_service,
        }
    }

    // ── Create ──────────────────────────────────────────────────

    /// Create a new escrow proposal.
    pub async fn create(
        &self,
        body: &CreateEscrowRequest,
        auth_headers: Option<&axum::http::HeaderMap>,
    ) -> Result<Escrow, ServiceError> {
        if body.amount_sompi <= 0 {
            return Err(ServiceError::InvalidInput("amount must be positive".into()));
        }

        // Optional auth: verify buyer address if headers present
        let buyer_address = if let Some(headers) = auth_headers {
            if headers.contains_key("x-daglock-address") {
                let auth = AuthContext::from_headers(headers)
                    .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
                if auth.address != body.buyer_address {
                    return Err(ServiceError::Forbidden(
                        "Auth address doesn't match buyer".into(),
                    ));
                }
                auth.address
            } else {
                body.buyer_address.clone()
            }
        } else {
            body.buyer_address.clone()
        };

        // Validate addresses
        let valid_addr = |a: &str| crate::api::escrows::validate_kaspa_address(a);
        if !valid_addr(&buyer_address) {
            return Err(ServiceError::InvalidInput("invalid buyer_address".into()));
        }
        if let Some(ref seller) = body.seller_address {
            if !valid_addr(seller) {
                return Err(ServiceError::InvalidInput("invalid seller_address".into()));
            }
        }

        // Validate trade_hash if provided
        if let Some(ref hash) = body.trade_hash {
            if !hash.is_empty() {
                daglock_shared::validate_trade_hash(hash)
                    .map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
            }
        }

        let escrow = Escrow {
            id: format!("esc_{}", uuid::Uuid::new_v4()),
            lock_tx_id: body.lock_tx_id.clone(),
            lock_tx_output_index: body.lock_tx_output_index,
            status: EscrowStatus::PendingConfirmation,
            asset_type: body.asset_type.clone().unwrap_or_else(|| "KAS".to_string()),
            buyer_address,
            seller_address: body.seller_address.clone(),
            amount_sompi: body.amount_sompi,
            fee_sompi: body.amount_sompi / daglock_shared::FEE_DENOMINATOR,
            template_hash: body.template_hash.clone().unwrap_or_default(),
            expiration_daa_score: body.expiration_daa_score,
            disputed_at: None,
            dispute_reason: None,
            cancelled_at: None,
            expired_at: None,
            created_at: chrono::Utc::now().timestamp(),
            settled_at: None,
            refunded_at: None,
            mediator_key: body.mediator_key.clone(),
            dispute_mode: body.dispute_mode.clone(),
            dispute_outcome: None,
            dispute_resolved_at: None,
            price_at_creation: body.price_at_creation,
            price_currency: body.price_currency.clone().or_else(|| {
                if body.price_type.as_deref() == Some("market") {
                    Some("USD".to_string())
                } else {
                    None
                }
            }),
            trade_hash: body.trade_hash.clone(),
            price_lock_time: if body.price_at_creation.is_some()
                || body.price_type.as_deref() == Some("market")
            {
                Some(chrono::Utc::now().timestamp())
            } else {
                None
            },
            price_at_settlement: if body.price_type.as_deref() == Some("market") {
                None
            } else {
                body.price_at_creation
            },
            price_source: if body.price_type.as_deref() == Some("market") {
                Some("coingecko".to_string())
            } else {
                None
            },
            price_type: body.price_type.clone(),
            invoice_id: body.invoice_id.clone(),
            memo: body.memo.clone(),
            auto_settle_timeout: body.auto_settle_timeout,
            mediation_status: None,
            mediation_buyer_claim: None,
            mediation_seller_claim: None,
            mediation_result: None,
            mediation_expires_at: None,
            mediation_buyer_accepted: None,
            mediation_seller_accepted: None,
            chat_pubkey_buyer: body.chat_pubkey.clone(),
            chat_pubkey_seller: None,
        };

        queries::insert_escrow(&self.db, &escrow)
            .await
            .map_err(|_| ServiceError::Internal("Failed to insert escrow".into()))?;

        let _ = self.ws_tx.send(WsEvent::escrow_created(&escrow.id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowCreated(&escrow.id));
        dispatch_email_notifications(&self.db, &self.email_service, &escrow, "created", "pending_confirmation").await;

        Ok(escrow)
    }

    // ── Settle ──────────────────────────────────────────────────

    /// Settle an escrow (buyer or seller, with auth).
    pub async fn settle(
        &self,
        id: &str,
        headers: &axum::http::HeaderMap,
    ) -> Result<(), ServiceError> {
        let current = self.get_active_escrow(id).await?;

        let auth = AuthContext::from_headers(headers)
            .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
        verify_settle_authorization(&current, &auth, self.sig_verifier.as_ref(), &self.db)
            .await
            .map_err(|e| ServiceError::Forbidden(e.to_string()))?;

        verify_escrow_active(&current, self.verifier.as_ref())
            .await
            .map_err(|e| ServiceError::VerificationFailed(e.to_string()))?;

        let settled = queries::settle_escrow_atomic(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to settle escrow".into()))?;

        if !settled {
            return Err(ServiceError::Conflict(
                "Escrow was already settled or is no longer active".into(),
            ));
        }

        let _ = self.ws_tx.send(WsEvent::escrow_settled(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowSettled(id));
        dispatch_email_notifications(&self.db, &self.email_service, &current, "settled", "settled").await;
        Ok(())
    }

    // ── Refund ──────────────────────────────────────────────────

    /// Refund an escrow (buyer only, with auth).
    pub async fn refund(
        &self,
        id: &str,
        headers: &axum::http::HeaderMap,
    ) -> Result<(), ServiceError> {
        let current = self.get_active_escrow(id).await?;

        let auth = AuthContext::from_headers(headers)
            .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
        verify_refund_authorization(&current, &auth, self.sig_verifier.as_ref(), &self.db)
            .await
            .map_err(|e| ServiceError::Forbidden(e.to_string()))?;

        verify_escrow_active(&current, self.verifier.as_ref())
            .await
            .map_err(|e| ServiceError::VerificationFailed(e.to_string()))?;

        let refunded = queries::refund_escrow_atomic(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to refund escrow".into()))?;

        if !refunded {
            return Err(ServiceError::Conflict(
                "Escrow was already refunded or is no longer active".into(),
            ));
        }

        let _ = self.ws_tx.send(WsEvent::escrow_refunded(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowRefunded(id));
        dispatch_email_notifications(&self.db, &self.email_service, &current, "refunded", "refunded").await;
        Ok(())
    }

    // ── Dispute ─────────────────────────────────────────────────

    /// Dispute an escrow. Optionally create a jury case.
    pub async fn dispute(
        &self,
        id: &str,
        reason: &str,
        mode: Option<&str>,
        headers: &axum::http::HeaderMap,
    ) -> Result<DisputeResponse, ServiceError> {
        let current = self.get_disputable_escrow(id).await?;

        let auth = AuthContext::from_headers(headers)
            .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
        let is_buyer = auth.address == current.buyer_address;
        let is_seller = current.seller_address.as_deref() == Some(&auth.address);
        if !is_buyer && !is_seller {
            return Err(ServiceError::Forbidden(
                "Only escrow parties can dispute".into(),
            ));
        }

        let parsed =
            parse_message(&auth.message).map_err(|e| ServiceError::InvalidInput(e.to_string()))?;
        if parsed.action != "dispute" || parsed.escrow_id != id {
            return Err(ServiceError::Forbidden(
                "Message must be 'dispute:{id}:ts:nonce'".into(),
            ));
        }
        if !self
            .sig_verifier
            .verify_signature(&auth.address, &auth.signature, &auth.message)
            .unwrap_or(false)
        {
            return Err(ServiceError::Forbidden("Invalid signature".into()));
        }
        verify_nonce(&self.db, &parsed, &auth.address)
            .await
            .map_err(|e| ServiceError::Forbidden(e.to_string()))?;

        queries::mark_escrow_disputed(&self.db, id, reason)
            .await
            .map_err(|_| ServiceError::Internal("Failed to mark escrow as disputed".into()))?;

        if mode == Some("jury") {
            let (juror_count, threshold) =
                crate::api::jury::juror_count_and_threshold(current.amount_sompi);

            let eligible = queries::list_eligible_jurors_simple(&self.db)
                .await
                .map_err(|_| ServiceError::Internal("Failed to list jurors".into()))?;

            if eligible.len() < juror_count as usize {
                return Err(ServiceError::Conflict(format!(
                    "Need {juror_count} jurors but only {} registered",
                    eligible.len()
                )));
            }

            let candidate_pool: Vec<_> = eligible
                .iter()
                .take((juror_count as usize).saturating_mul(2).min(eligible.len()))
                .collect();
            let pool_size = candidate_pool.len();
            let needed = (juror_count as usize).min(pool_size);
            let mut indices: Vec<usize> = (0..pool_size).collect();
            for i in (pool_size - needed..pool_size).rev() {
                let j = rand::random::<usize>() % (i + 1);
                indices.swap(i, j);
            }
            let selected: Vec<String> = indices[pool_size - needed..]
                .iter()
                .map(|&i| candidate_pool[i].address.clone())
                .collect();

            let case_id =
                queries::create_jury_case(&self.db, id, juror_count, threshold, &selected)
                    .await
                    .map_err(|_| ServiceError::Internal("Failed to create jury case".into()))?;

            Ok(DisputeResponse::Jury {
                case_id,
                juror_count,
                threshold,
            })
        } else {
            let _ = self.ws_tx.send(WsEvent::escrow_disputed(id, reason));
            Ok(DisputeResponse::Standard)
        }
    }

    // ── Cancel ──────────────────────────────────────────────────

    /// Cancel an escrow (creator only, with auth).
    pub async fn cancel(
        &self,
        id: &str,
        headers: &axum::http::HeaderMap,
    ) -> Result<(), ServiceError> {
        let current = self.get_active_escrow(id).await?;

        let auth = AuthContext::from_headers(headers)
            .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
        verify_cancel_authorization(&current, &auth, self.sig_verifier.as_ref(), &self.db)
            .await
            .map_err(|e| ServiceError::Forbidden(e.to_string()))?;

        queries::mark_escrow_cancelled(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to cancel escrow".into()))?;

        let _ = self.ws_tx.send(WsEvent::escrow_cancelled(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowCancelled(id));
        Ok(())
    }

    // ── Atomic Swap ─────────────────────────────────────────────

    /// Settle an escrow via atomic swap (preimage verification).
    pub async fn atomic_swap(&self, id: &str, preimage_hex: &str) -> Result<(), ServiceError> {
        let current = self.get_settleable_escrow(id).await?;

        let preimage_bytes = hex::decode(preimage_hex)
            .map_err(|_| ServiceError::InvalidInput("Preimage must be valid hex".into()))?;

        if preimage_bytes.is_empty() || preimage_bytes.len() > 1024 {
            return Err(ServiceError::InvalidInput(
                "Preimage must be 1-1024 bytes".into(),
            ));
        }

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&preimage_bytes);
        let hash = hasher.finalize();
        let hash_hex = hex::encode(hash);

        if let Some(ref expected) = current.trade_hash {
            if !expected.is_empty() && hash_hex != *expected {
                return Err(ServiceError::Forbidden(
                    "Preimage does not match trade hash".into(),
                ));
            }
        }

        let settled = queries::settle_escrow_atomic(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to settle escrow".into()))?;

        if !settled {
            return Err(ServiceError::Conflict("Escrow was already settled".into()));
        }

        let _ = self.ws_tx.send(WsEvent::escrow_settled(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowSettled(id));
        Ok(())
    }

    // ── Auto-Settle ─────────────────────────────────────────────

    /// Auto-settle an escrow after timeout. No auth required — the
    /// covenant's auto_settle() entrypoint enforces the timeout on-chain.
    pub async fn auto_settle(&self, id: &str) -> Result<(), ServiceError> {
        let current = self.get_settleable_escrow(id).await?;

        let now = chrono::Utc::now().timestamp();
        let timeout = current
            .auto_settle_timeout
            .ok_or_else(|| ServiceError::InvalidInput("Escrow has no auto-settle timeout".into()))?;

        if now < timeout {
            return Err(ServiceError::Forbidden(
                "Auto-settle timeout has not elapsed yet".into(),
            ));
        }

        let settled = queries::auto_settle_escrow_atomic(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to auto-settle escrow".into()))?;

        if !settled {
            return Err(ServiceError::Conflict(
                "Escrow could not be auto-settled (not active or timeout not reached)".into(),
            ));
        }

        let _ = self.ws_tx.send(WsEvent::escrow_settled(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowSettled(id));
        dispatch_email_notifications(
            &self.db,
            &self.email_service,
            &current,
            "settled",
            "settled",
        )
        .await;
        Ok(())
    }

    // ── Mediation Outcome Execution ─────────────────────────────

    /// Force settle a disputed escrow (mediation accepted: payout outcome).
    pub async fn force_settle(&self, id: &str) -> Result<(), ServiceError> {
        let settled = queries::force_settle_disputed(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to force settle escrow".into()))?;
        if !settled {
            return Err(ServiceError::Conflict(
                "Escrow is not in disputed state".into(),
            ));
        }
        let _ = self.ws_tx.send(WsEvent::escrow_settled(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowSettled(id));
        Ok(())
    }

    /// Force refund a disputed escrow (mediation accepted: refund outcome).
    pub async fn force_refund(&self, id: &str) -> Result<(), ServiceError> {
        let refunded = queries::force_refund_disputed(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to force refund escrow".into()))?;
        if !refunded {
            return Err(ServiceError::Conflict(
                "Escrow is not in disputed state".into(),
            ));
        }
        let _ = self.ws_tx.send(WsEvent::escrow_refunded(id));
        webhooks::dispatch(self.db.clone(), WebhookEvent::EscrowRefunded(id));
        Ok(())
    }

    // ── Internal helpers ────────────────────────────────────────

    /// Get an escrow that can be settled or refunded (active or pending_confirmation).
    async fn get_settleable_escrow(&self, id: &str) -> Result<Escrow, ServiceError> {
        let escrow = queries::get_escrow(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to query escrow".into()))?
            .ok_or_else(|| ServiceError::NotFound(format!("No escrow found with id '{id}'")))?;

        match escrow.status {
            EscrowStatus::Settled
            | EscrowStatus::Refunded
            | EscrowStatus::Cancelled
            | EscrowStatus::Expired => {
                Err(ServiceError::Conflict("Escrow is already finalized".into()))
            }
            _ => Ok(escrow),
        }
    }

    /// Get an escrow that can be acted upon (active or pending_confirmation).
    async fn get_active_escrow(&self, id: &str) -> Result<Escrow, ServiceError> {
        self.get_settleable_escrow(id).await
    }

    /// Get an escrow that can be disputed (active, pending_confirmation, not yet disputed).
    async fn get_disputable_escrow(&self, id: &str) -> Result<Escrow, ServiceError> {
        let escrow = self.get_active_escrow(id).await?;
        if escrow.status == EscrowStatus::Disputed {
            return Err(ServiceError::Conflict("Escrow is already disputed".into()));
        }
        Ok(escrow)
    }
}

/// Response variant for dispute operations.
#[allow(dead_code)]
pub enum DisputeResponse {
    Standard,
    Jury {
        case_id: String,
        juror_count: i64,
        threshold: i64,
    },
}

/// Dispatch email notifications for an escrow event.
/// Queries the DB for verified email subscribers who opted into this event type,
/// and sends them a notification via the EmailService.
pub async fn dispatch_email_notifications(
    db: &Pool<Sqlite>,
    email_service: &Option<std::sync::Arc<crate::services::email::EmailService>>,
    escrow: &Escrow,
    event_type: &str,
    status: &str,
) {
    let email_service = match email_service {
        Some(s) => s,
        None => return,
    };
    if !email_service.is_configured() {
        return;
    }

    let event_column = match event_type {
        "created" => "notify_created",
        "settled" => "notify_settled",
        "disputed" => "notify_disputed",
        "refunded" => "notify_refunded",
        "expired" => "notify_expired",
        _ => return,
    };

    // Fetch subscribers for both buyer and seller
    let subscribers = match queries::get_verified_subscribers_for_event(db, event_column).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to fetch email subscribers: {e}");
            return;
        }
    };

    // Filter subscribers who are involved in this escrow
    for sub in &subscribers {
        if sub.address != escrow.buyer_address
            && sub.address != escrow.seller_address.as_deref().unwrap_or("")
        {
            continue;
        }

        if let Err(e) = email_service
            .notify_escrow_event(
                &sub.email,
                &sub.address,
                event_type,
                &escrow.id,
                escrow.amount_sompi,
                status,
            )
            .await
        {
            tracing::warn!("Failed to send email notification to {}: {e}", sub.email);
        }
    }
}

// validate_kaspa_address removed — use daglock_shared::validate_kaspa_address instead
