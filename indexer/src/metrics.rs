//! Prometheus metrics for DagLock indexer.
//!
//! Exposes a /v1/metrics endpoint with counters for requests,
//! escrow lifecycle events, and operational health signals.

use axum::Json;
use once_cell::sync::Lazy;
use prometheus::{
    register_counter, register_gauge, register_histogram_vec, Counter, Gauge, HistogramVec,
};
use serde_json::{json, Value};

/// HTTP request counter labelled by method and path.
pub static HTTP_REQUESTS: Lazy<Counter> = Lazy::new(|| {
    register_counter!("daglock_http_requests_total", "Total HTTP requests")
        .expect("HTTP_REQUESTS metric already registered")
});

/// Escrow lifecycle counters labelled by action (created, settled, refunded, disputed, cancelled).
pub static ESCROW_EVENTS: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "daglock_escrow_events_total",
        "Escrow lifecycle events by type"
    )
    .expect("ESCROW_EVENTS metric already registered")
});

/// Active escrows gauge.
pub static ACTIVE_ESCROWS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "daglock_active_escrows",
        "Number of currently active escrows"
    )
    .expect("ACTIVE_ESCROWS metric already registered")
});

/// WebSocket connections gauge.
pub static WS_CONNECTIONS: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "daglock_ws_connections",
        "Number of connected WebSocket clients"
    )
    .expect("WS_CONNECTIONS metric already registered")
});

/// Database query duration histogram.
pub static DB_QUERY_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "daglock_db_query_duration_seconds",
        "Database query duration in seconds",
        &["query"]
    )
    .expect("DB_QUERY_DURATION metric already registered")
});

/// Request duration histogram.
pub static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "daglock_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "path"]
    )
    .expect("REQUEST_DURATION metric already registered")
});

/// Full prometheus text format output.
pub fn render_metrics() -> String {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        tracing::warn!("Failed to encode metrics: {e}");
        return String::new();
    }
    String::from_utf8(buffer).unwrap_or_default()
}

/// GET /v1/metrics — Prometheus text format
pub async fn metrics_handler() -> String {
    render_metrics()
}

/// GET /v1/metrics/json — JSON format for debugging
pub async fn metrics_json() -> Json<Value> {
    Json(json!({
        "http_requests_total": HTTP_REQUESTS.get(),
        "escrow_events_total": ESCROW_EVENTS.get(),
        "active_escrows": ACTIVE_ESCROWS.get(),
        "ws_connections": WS_CONNECTIONS.get(),
    }))
}

// ponytail: prometheus encoder panics if no metrics registered.
// Upgrade path: register all intended metrics at startup in main().
