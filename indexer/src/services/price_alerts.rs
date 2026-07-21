use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::services::email::EmailService;
use crate::types::PriceAlert;
use crate::websocket::WsEvent;

/// Check all active (non-triggered) price alerts against the current price.
/// Marks triggered alerts and sends email + WebSocket notifications.
pub async fn check_alerts(
    pool: &Pool<Sqlite>,
    current_price: f64,
    ws_tx: &broadcast::Sender<WsEvent>,
    email_service: &Option<Arc<EmailService>>,
) {
    let alerts = match get_active_alerts(pool).await {
        Ok(a) => a,
        Err(e) => {
            error!("Price alerts: failed to fetch active alerts: {e}");
            return;
        }
    };

    if alerts.is_empty() {
        return;
    }

    let now = chrono::Utc::now().timestamp();

    for alert in &alerts {
        let triggered = match alert.direction.as_str() {
            "above" => current_price >= alert.target_price,
            "below" => current_price <= alert.target_price,
            _ => false,
        };

        if !triggered {
            continue;
        }

        // Mark alert as triggered
        if let Err(e) = mark_triggered(pool, &alert.id, now).await {
            warn!(
                "Price alerts: failed to mark alert {} triggered: {e}",
                alert.id
            );
            continue;
        }

        info!(
            "Price alert {} triggered: KAS/USD ${:.4} is {} ${:.4}",
            alert.id, current_price, alert.direction, alert.target_price
        );

        // WebSocket notification
        let _ = ws_tx.send(WsEvent {
            event: "price_alert_triggered".to_string(),
            data: serde_json::json!({
                "alert_id": alert.id,
                "target_price": alert.target_price,
                "direction": alert.direction,
                "current_price": current_price,
            }),
        });

        // Email notification
        if let Some(ref svc) = email_service {
            if svc.is_configured() {
                let subject = format!(
                    "[DagLock] Price Alert: KAS is {} ${:.4}",
                    if alert.direction == "above" {
                        "above"
                    } else {
                        "below"
                    },
                    alert.target_price
                );
                let body = format!(
                    "DagLock Price Alert\n\
                     ===================\n\n\
                     KAS/USD is now ${:.4}\n\
                     Your alert: notify when price goes {} ${:.4}\n\n\
                     ---\n\
                     DagLock — Trustless Escrow on Kaspa\n\
                     https://daglock.com",
                    current_price, alert.direction, alert.target_price
                );
                if let Err(e) = svc.send_notification(&alert.address, &subject, &body).await {
                    warn!(
                        "Price alerts: failed to send email for alert {}: {e}",
                        alert.id
                    );
                }
            }
        }
    }
}

async fn get_active_alerts(pool: &Pool<Sqlite>) -> Result<Vec<PriceAlert>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, f64, String, i32, i64, Option<i64>)>(
        "SELECT id, address, target_price, direction, triggered, created_at, triggered_at \
         FROM price_alerts WHERE triggered = 0",
    )
    .fetch_all(pool)
    .await?;

    let alerts = rows
        .into_iter()
        .map(
            |(id, address, target_price, direction, triggered, created_at, triggered_at)| {
                PriceAlert {
                    id,
                    address,
                    target_price,
                    direction,
                    triggered: triggered != 0,
                    created_at,
                    triggered_at,
                }
            },
        )
        .collect();

    Ok(alerts)
}

async fn mark_triggered(pool: &Pool<Sqlite>, alert_id: &str, now: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE price_alerts SET triggered = 1, triggered_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(alert_id)
        .execute(pool)
        .await?;
    Ok(())
}
