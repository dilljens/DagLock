//! Simple per-IP request rate limiter using axum::middleware::from_fn.

use axum::{
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{collections::HashMap, net::IpAddr, sync::Mutex, time::Instant};

/// Thread-safe rate limiter state.
pub struct RateLimiter {
    inner: Mutex<HashMap<IpAddr, WindowState>>,
    max_requests: u32,
    window_secs: u64,
}

struct WindowState {
    count: u32,
    reset_at: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    pub fn check(&self, ip: IpAddr) -> std::result::Result<(), Response> {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();

        if let Some(entry) = map.get_mut(&ip) {
            if now >= entry.reset_at {
                entry.count = 1;
                entry.reset_at = now + std::time::Duration::from_secs(self.window_secs);
                Ok(())
            } else if entry.count < self.max_requests {
                entry.count += 1;
                Ok(())
            } else {
                Err((
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "rate_limited",
                        "message": format!(
                            "Rate limit exceeded. Max {} requests per {} seconds.",
                            self.max_requests, self.window_secs
                        ),
                    })),
                )
                    .into_response())
            }
        } else {
            map.insert(
                ip,
                WindowState {
                    count: 1,
                    reset_at: now + std::time::Duration::from_secs(self.window_secs),
                },
            );
            Ok(())
        }
    }
}

/// Axum middleware function for rate limiting.
pub async fn rate_limit_mw(
    state: axum::extract::State<std::sync::Arc<RateLimiter>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Extract IP from X-Forwarded-For or remote addr
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    match state.check(ip) {
        Ok(()) => next.run(req).await,
        Err(resp) => resp,
    }
}
