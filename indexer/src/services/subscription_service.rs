//! Subscription business logic — recurring payments.
//!
//! `SubscriptionService` owns subscription lifecycle: create, cancel, draw.
//! Handlers delegate to this layer for validation, auth checks, and DB updates.

use crate::auth::{AuthContext, SignatureVerifier};
use crate::db::queries;
use crate::types::*;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;

/// Error type returned by service methods.
#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    InvalidInput(String),
    Unauthorized(String),
    Forbidden(String),
    Conflict(String),
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
            Self::Internal(msg) => write!(f, "{msg}"),
        }
    }
}

impl ServiceError {
    pub fn error_code(&self) -> &str {
        match self {
            Self::NotFound(_) => "subscription_not_found",
            Self::InvalidInput(_) => "invalid_input",
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Conflict(_) => "subscription_already_finalized",
            Self::Internal(_) => "internal_error",
        }
    }
}

/// Subscription lifecycle service.
pub struct SubscriptionService {
    db: Pool<Sqlite>,
    sig_verifier: Arc<dyn SignatureVerifier>,
}

#[allow(dead_code)]
impl SubscriptionService {
    pub fn new(
        db: Pool<Sqlite>,
        sig_verifier: Arc<dyn SignatureVerifier>,
    ) -> Self {
        Self { db, sig_verifier }
    }

    /// Create a new subscription record.
    pub async fn create(
        &self,
        body: &CreateSubscriptionRequest,
        auth_headers: Option<&axum::http::HeaderMap>,
    ) -> Result<Subscription, ServiceError> {
        if body.total_amount <= 0 {
            return Err(ServiceError::InvalidInput("total_amount must be positive".into()));
        }
        if body.installment_amount <= 0 {
            return Err(ServiceError::InvalidInput("installment_amount must be positive".into()));
        }
        if body.installment_amount > body.total_amount {
            return Err(ServiceError::InvalidInput(
                "installment_amount cannot exceed total_amount".into(),
            ));
        }
        if body.interval_seconds <= 0 {
            return Err(ServiceError::InvalidInput("interval_seconds must be positive".into()));
        }
        if body.max_periods <= 0 {
            return Err(ServiceError::InvalidInput("max_periods must be positive".into()));
        }

        let valid_addr = |a: &str| crate::api::escrows::validate_kaspa_address(a);
        if !valid_addr(&body.payer_address) {
            return Err(ServiceError::InvalidInput("invalid payer_address".into()));
        }
        if !valid_addr(&body.recipient_address) {
            return Err(ServiceError::InvalidInput("invalid recipient_address".into()));
        }
        if body.payer_address == body.recipient_address {
            return Err(ServiceError::InvalidInput("payer and recipient cannot be the same".into()));
        }

        if let Some(headers) = auth_headers {
            if headers.contains_key("x-daglock-address") {
                let auth = AuthContext::from_headers(headers)
                    .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;
                if auth.address != body.payer_address {
                    return Err(ServiceError::Forbidden(
                        "Signed address must match payer_address".into(),
                    ));
                }
            }
        }

        let sub = Subscription {
            id: body.id.clone().unwrap_or_else(|| {
                format!("sub_{}", uuid::Uuid::new_v4())
            }),
            payer_address: body.payer_address.clone(),
            recipient_address: body.recipient_address.clone(),
            total_amount: body.total_amount,
            installment_amount: body.installment_amount,
            interval_seconds: body.interval_seconds,
            start_time: body.start_time,
            current_period: body.current_period.unwrap_or(0),
            max_periods: body.max_periods,
            status: "active".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            cancelled_at: None,
            completed_at: None,
        };

        queries::subscriptions::insert_subscription(&self.db, &sub)
            .await
            .map_err(|_| ServiceError::Internal("Failed to insert subscription".into()))?;

        Ok(sub)
    }

    /// Cancel an active subscription (payer only).
    pub async fn cancel(
        &self,
        id: &str,
        headers: &axum::http::HeaderMap,
    ) -> Result<(), ServiceError> {
        let sub = self.get_active_subscription(id).await?;

        let auth = AuthContext::from_headers(headers)
            .map_err(|e| ServiceError::Unauthorized(e.to_string()))?;

        if auth.address != sub.payer_address {
            return Err(ServiceError::Forbidden(
                "Only the payer can cancel a subscription".into(),
            ));
        }

        if !self
            .sig_verifier
            .verify_signature(&auth.address, &auth.signature, &auth.message)
            .unwrap_or(false)
        {
            return Err(ServiceError::Forbidden("Invalid signature".into()));
        }

        queries::subscriptions::mark_subscription_cancelled(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to cancel subscription".into()))?;

        Ok(())
    }

    /// Draw the current installment. Anyone can trigger this — the covenant
    /// enforces recipient signature on-chain.
    pub async fn draw(&self, id: &str) -> Result<Subscription, ServiceError> {
        let sub = self.get_active_subscription(id).await?;

        let now = chrono::Utc::now().timestamp();
        let next_due = sub.start_time + sub.current_period * sub.interval_seconds;
        if now < next_due {
            return Err(ServiceError::Forbidden(format!(
                "Installment not due yet. Next draw at {next_due}"
            )));
        }

        if sub.current_period >= sub.max_periods {
            return Err(ServiceError::Conflict("All installments have been drawn".into()));
        }

        let advanced = queries::subscriptions::advance_subscription_period(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to advance subscription period".into()))?;

        if !advanced {
            return Err(ServiceError::Conflict("Subscription could not be advanced".into()));
        }

        let updated = queries::subscriptions::get_subscription(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to fetch updated subscription".into()))?
            .ok_or_else(|| ServiceError::Internal("Subscription disappeared after draw".into()))?;

        // If all periods are drawn, mark as completed
        if updated.current_period >= updated.max_periods {
            let _ = queries::subscriptions::mark_subscription_completed(&self.db, id).await;
        }

        Ok(updated)
    }

    /// Get an active subscription.
    async fn get_active_subscription(&self, id: &str) -> Result<Subscription, ServiceError> {
        let sub = queries::subscriptions::get_subscription(&self.db, id)
            .await
            .map_err(|_| ServiceError::Internal("Failed to query subscription".into()))?
            .ok_or_else(|| ServiceError::NotFound(format!("No subscription found with id '{id}'")))?;

        if sub.status != "active" {
            return Err(ServiceError::Conflict(format!(
                "Subscription is {}. Only active subscriptions can be modified",
                sub.status
            )));
        }

        Ok(sub)
    }
}
