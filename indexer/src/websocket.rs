//! WebSocket handler for real-time escrow updates.
//!
//! Provides a WebSocket endpoint for clients to subscribe to
//! escrow lifecycle events. Requires authentication via query params
//! (X-Daglock-Address, X-Daglock-Signature, X-Daglock-Message).
//! Only events for escrows the caller is a participant in are delivered.

use axum::extract::ws::{Message, WebSocket};
use serde_json::json;
use sqlx::{Pool, Sqlite};
use std::collections::HashSet;
use tokio::sync::broadcast;
use tracing::{info, warn};

/// WebSocket event types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WsEvent {
    pub event: String,
    pub data: serde_json::Value,
}

impl WsEvent {
    pub fn escrow_created(escrow_id: &str) -> Self {
        Self {
            event: "escrow_created".to_string(),
            data: json!({ "escrow_id": escrow_id }),
        }
    }

    pub fn escrow_settled(escrow_id: &str) -> Self {
        Self {
            event: "escrow_settled".to_string(),
            data: json!({ "escrow_id": escrow_id }),
        }
    }

    pub fn escrow_refunded(escrow_id: &str) -> Self {
        Self {
            event: "escrow_refunded".to_string(),
            data: json!({ "escrow_id": escrow_id }),
        }
    }

    pub fn escrow_expired(escrow_id: &str) -> Self {
        Self {
            event: "escrow_expired".to_string(),
            data: json!({ "escrow_id": escrow_id }),
        }
    }

    pub fn escrow_cancelled(escrow_id: &str) -> Self {
        Self {
            event: "escrow_cancelled".to_string(),
            data: json!({ "escrow_id": escrow_id }),
        }
    }

    pub fn escrow_disputed(escrow_id: &str, reason: &str) -> Self {
        Self {
            event: "escrow_disputed".to_string(),
            data: json!({ "escrow_id": escrow_id, "reason": reason }),
        }
    }

    pub fn offer_created(offer_id: &str) -> Self {
        Self {
            event: "offer_created".to_string(),
            data: json!({ "offer_id": offer_id }),
        }
    }

    pub fn offer_accepted(offer_id: &str) -> Self {
        Self {
            event: "offer_accepted".to_string(),
            data: json!({ "offer_id": offer_id }),
        }
    }

    pub fn offer_cancelled(offer_id: &str) -> Self {
        Self {
            event: "offer_cancelled".to_string(),
            data: json!({ "offer_id": offer_id }),
        }
    }
}

/// Max incoming WebSocket message size in bytes (256 KB).
const MAX_WS_MESSAGE_SIZE: usize = 256 * 1024;

/// Handle a WebSocket connection.
///
/// If `auth_addresses` is provided, only events for escrows where the
/// authenticated user is a participant are forwarded. Without it (unauthenticated),
/// no events are forwarded for privacy.
pub async fn handle_socket(
    mut socket: WebSocket,
    pool: Pool<Sqlite>,
    mut rx: broadcast::Receiver<WsEvent>,
    auth_addresses: Option<HashSet<String>>,
) {
    info!("WebSocket client connected (authenticated: {})", auth_addresses.is_some());

    // Send initial connection message
    let msg = WsEvent {
        event: "connected".to_string(),
        data: json!({ "message": "Connected to DagLock real-time updates" }),
    };
    if let Ok(json_str) = serde_json::to_string(&msg) {
        if let Err(e) = socket.send(Message::Text(json_str)).await {
            warn!("Failed to send connection message: {}", e);
            return;
        }
    }

    // Read loop: discard incoming messages (fire-and-forget event bus),
    // but enforce max size to prevent resource exhaustion.
    // In the future this could handle client-side pings or subscriptions.
    loop {
        tokio::select! {
            // Check for incoming client messages (with size limit)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(t))) if t.len() > MAX_WS_MESSAGE_SIZE => {
                        warn!("WebSocket: client sent oversized message ({} bytes), closing", t.len());
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(Message::Binary(b))) if b.len() > MAX_WS_MESSAGE_SIZE => {
                        warn!("WebSocket: client sent oversized binary message ({} bytes), closing", b.len());
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = socket.send(Message::Pong(data)).await {
                            warn!("WebSocket pong error: {}", e);
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("WebSocket client disconnected (close frame)");
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket recv error: {}", e);
                        break;
                    }
                    _ => {} // Text/Binary under limit or other — ignore
                }
            }
            // Forward broadcast events to client, filtered by address
            event = rx.recv() => {
                let event = match event {
                    Ok(e) => e,
                    Err(_) => break,
                };
        // If no auth, don't forward any events (privacy)
        let addresses = match &auth_addresses {
            Some(a) => a,
            None => continue,
        };

        // Check if this event involves one of the caller's escrows
        let escrow_id = event.data.get("escrow_id").and_then(|v| v.as_str()).unwrap_or("");
        if !escrow_id.is_empty() {
            // Check DB to see if caller is a participant in this escrow
            match sqlx::query_as::<_, (String,)>(
                "SELECT buyer_address FROM escrows WHERE id = ?1"
            )
            .bind(escrow_id)
            .fetch_optional(&pool)
            .await
            {
                Ok(Some((buyer,))) if addresses.contains(&buyer) => {
                    // Forward to authenticated participant
                }
                Ok(_) => {
                    // Also check seller
                    match sqlx::query_as::<_, (Option<String>,)>(
                        "SELECT seller_address FROM escrows WHERE id = ?1"
                    )
                    .bind(escrow_id)
                    .fetch_optional(&pool)
                    .await
                    {
                        Ok(Some((Some(seller),))) if addresses.contains(&seller) => {
                            // Forward to authenticated participant
                        }
                        _ => continue, // Skip — caller is not a participant
                    }
                }
                Err(_) => continue,
            }
        }

        if let Ok(json_str) = serde_json::to_string(&event) {
            if let Err(e) = socket.send(Message::Text(json_str)).await {
                warn!("WebSocket send error: {}", e);
                break;
            }
        }
            } // end of event = rx.recv() arm
        } // end of tokio::select!
    } // end of loop

    info!("WebSocket client disconnected");
}

/// Create a broadcast channel for WebSocket events.
#[allow(dead_code)]
pub fn create_event_channel() -> broadcast::Sender<WsEvent> {
    let (tx, _) = broadcast::channel(4096);
    tx
}


