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

#[allow(dead_code)]
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
    if let Err(e) = socket
        .send(Message::Text(serde_json::to_string(&msg).unwrap()))
        .await
    {
        warn!("Failed to send connection message: {}", e);
        return;
    }

    // Forward broadcast events to WebSocket client
    while let Ok(event) = rx.recv().await {
        let json = serde_json::to_string(&event).unwrap();
        if let Err(e) = socket.send(Message::Text(json)).await {
            warn!("WebSocket send error: {}", e);
            break;
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

/// Broadcast an event to all connected WebSocket clients.
#[allow(dead_code)]
pub fn broadcast_event(tx: &broadcast::Sender<WsEvent>, event: WsEvent) {
    // Ignore broadcast errors (no receivers is fine)
    let _ = tx.send(event);
}
