#![allow(dead_code)]
//! Webhook delivery service.
//!
//! Dispatches lifecycle events to registered webhook endpoints.
//! Retries up to 3 times with exponential backoff (1s, 4s, 10s).

use sqlx::{Pool, Sqlite};
use tracing::{error, info, warn};

/// Event types that can trigger webhooks.
#[derive(Debug, Clone, Copy)]
pub enum WebhookEvent<'a> {
    EscrowCreated(&'a str),
    EscrowSettled(&'a str),
    EscrowRefunded(&'a str),
    EscrowDisputed(&'a str),
    EscrowCancelled(&'a str),
    EscrowExpired(&'a str),
    OfferCreated(&'a str),
    OfferAccepted(&'a str),
}

impl WebhookEvent<'_> {
    fn event_name(&self) -> &'static str {
        match self {
            Self::EscrowCreated(_) => "escrow.created",
            Self::EscrowSettled(_) => "escrow.settled",
            Self::EscrowRefunded(_) => "escrow.refunded",
            Self::EscrowDisputed(_) => "escrow.disputed",
            Self::EscrowCancelled(_) => "escrow.cancelled",
            Self::EscrowExpired(_) => "escrow.expired",
            Self::OfferCreated(_) => "offer.created",
            Self::OfferAccepted(_) => "offer.accepted",
        }
    }

    fn id(&self) -> &str {
        match self {
            Self::EscrowCreated(id)
            | Self::EscrowSettled(id)
            | Self::EscrowRefunded(id)
            | Self::EscrowDisputed(id)
            | Self::EscrowCancelled(id)
            | Self::EscrowExpired(id)
            | Self::OfferCreated(id)
            | Self::OfferAccepted(id) => id,
        }
    }

    fn payload_json(&self) -> String {
        let event = self.event_name();
        let id = self.id();
        serde_json::json!({
            "event": event,
            "created_at": chrono::Utc::now().timestamp(),
            "data": { "id": id }
        })
        .to_string()
    }
}

/// Dispatch a webhook event to all subscribers.
/// Runs asynchronously — does not block the caller.
pub fn dispatch(pool: Pool<Sqlite>, event: WebhookEvent<'_>) {
    let event_name = event.event_name().to_string();
    let payload = event.payload_json();
    let escrow_id = event.id().to_string();

    tokio::spawn(async move {
        // Only dispatch webhooks for apps that have at least one active API key
        // with webhooks_enabled = 1 (free tier = no webhooks, pro+ = yes).
        let hooks = match sqlx::query_as::<_, (String, String)>(
            "SELECT w.id, w.url FROM webhooks w
             INNER JOIN apps a ON a.id = w.app_id
             WHERE w.event = ?1 AND w.is_active = 1 AND a.is_active = 1
             AND EXISTS (
                 SELECT 1 FROM api_keys k
                 WHERE k.app_id = a.id AND k.is_active = 1 AND k.webhooks_enabled = 1
             )",
        )
        .bind(&event_name)
        .fetch_all(&pool)
        .await
        {
            Ok(h) => h,
            Err(e) => {
                error!("Webhook query failed for {event_name}: {e}");
                return;
            }
        };

        if hooks.is_empty() {
            info!("No webhook subscribers for {event_name} ({escrow_id})");
            return;
        }

        let client = reqwest::Client::new();
        for (hook_id, url) in &hooks {
            info!("Delivering webhook {event_name} to {url}");

            let delivery_id = format!("whd_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
            let now = chrono::Utc::now().timestamp();

            // Record delivery attempt
            let _ = sqlx::query(
                "INSERT INTO webhook_deliveries (id, webhook_id, event, payload, status, attempts, created_at, next_retry_at)
                 VALUES (?1, ?2, ?3, ?4, 'pending', 0, ?5, ?5)"
            )
            .bind(&delivery_id).bind(hook_id).bind(&event_name).bind(&payload).bind(now)
            .execute(&pool).await;

            // Send with retries
            for attempt in 0..3 {
                let delay = match attempt {
                    0 => 0,
                    1 => 1,
                    _ => 4,
                };
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;

                match client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .header("X-Daglock-Webhook-Id", &delivery_id)
                    .header("X-Daglock-Webhook-Timestamp", now.to_string())
                    .body(payload.clone())
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let code = resp.status().as_u16() as i64;
                        let _ = sqlx::query(
                            "UPDATE webhook_deliveries SET status = CASE WHEN ?2 BETWEEN 200 AND 299 THEN 'delivered' ELSE 'failed' END,
                             attempts = attempts + 1, response_code = ?2, last_attempt_at = ?3
                             WHERE id = ?1"
                        )
                        .bind(&delivery_id).bind(code).bind(chrono::Utc::now().timestamp())
                        .execute(&pool).await;

                        if (200..300).contains(&resp.status().as_u16()) {
                            info!("Webhook {delivery_id} delivered to {url} (HTTP {code})");
                            break;
                        }
                        warn!(
                            "Webhook {delivery_id} got HTTP {code} from {url} (attempt {})",
                            attempt + 1
                        );
                    }
                    Err(e) => {
                        let _ = sqlx::query(
                            "UPDATE webhook_deliveries SET status = 'failed', attempts = attempts + 1, last_attempt_at = ?2
                             WHERE id = ?1"
                        )
                        .bind(&delivery_id).bind(chrono::Utc::now().timestamp())
                        .execute(&pool).await;
                        warn!(
                            "Webhook {delivery_id} delivery error: {e} (attempt {})",
                            attempt + 1
                        );
                    }
                }
            }
        }
    });
}
