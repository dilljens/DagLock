//! WebSocket handler for real-time escrow updates.
//!
//! Provides a WebSocket endpoint for clients to subscribe to
//! escrow lifecycle events.

use axum::extract::ws::{Message, WebSocket};
use serde_json::json;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;
use tracing::{info, warn};

/// WebSocket event types.
#[derive(Debug, Clone, serde::Serialize)]
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

/// Handle a WebSocket connection.
pub async fn handle_socket(
    mut socket: WebSocket,
    _pool: Pool<Sqlite>,
    mut rx: broadcast::Receiver<WsEvent>,
) {
    info!("WebSocket client connected");

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

    // Forward broadcast events to WebSocket client
    while let Ok(event) = rx.recv().await {
        if let Ok(json_str) = serde_json::to_string(&event) {
            if let Err(e) = socket.send(Message::Text(json_str)).await {
                warn!("WebSocket send error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket client disconnected");
}

/// Create a broadcast channel for WebSocket events.
#[allow(dead_code)]
pub fn create_event_channel() -> broadcast::Sender<WsEvent> {
    let (tx, _) = broadcast::channel(100);
    tx
}


