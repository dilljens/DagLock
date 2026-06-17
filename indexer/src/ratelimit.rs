//! Simple per-IP request rate limiter using axum::middleware::from_fn.
//!
//! Tracks requests per IP with a sliding window counter.
//! Returns HTTP 429 Too Many Requests when exceeded.

use axum::{
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{collections::HashMap, net::IpAddr, sync::Mutex, time::Instant};

/// Rate limit tiers.
const DEFAULT_MAX: u32 = 30;
const API_KEY_MAX: u32 = 300;
const WINDOW_SECS: u64 = 60;

/// Thread-safe per-IP rate limiter.
pub struct RateLimiter {
    inner: Mutex<HashMap<IpAddr, WindowState>>,
}

struct WindowState {
    count: u32,
    reset_at: Instant,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Check if an IP is within its rate limit.
    /// `max_requests` overrides the default limit (e.g., 300 for API key holders).
    pub fn check(&self, ip: IpAddr, max_requests: u32) -> std::result::Result<(), Response> {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();

        if let Some(entry) = map.get_mut(&ip) {
            if now >= entry.reset_at {
                entry.count = 1;
                entry.reset_at = now + std::time::Duration::from_secs(WINDOW_SECS);
                entry.max_requests = max_requests;
                Ok(())
            } else if entry.count < entry.max_requests {
                entry.count += 1;
                Ok(())
            } else {
                Err((
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "rate_limited",
                        "message": format!(
                            "Rate limit exceeded. Max {} requests per {} seconds.",
                            entry.max_requests, WINDOW_SECS
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
                    reset_at: now + std::time::Duration::from_secs(WINDOW_SECS),
                    max_requests,
                },
            );
            Ok(())
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Axum middleware function for rate limiting.
/// Checks for X-Daglock-Api-Key header — if present, allows 300 req/min instead of 30.
pub async fn rate_limit_mw(
    state: axum::extract::State<std::sync::Arc<RateLimiter>>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Response {
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .and_then(|v| v.trim().parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

    // API key holders get 10x the rate limit
    let has_api_key = req
        .headers()
        .get("x-daglock-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    let max_requests = if has_api_key {
        API_KEY_MAX
    } else {
        DEFAULT_MAX
    };

    match state.check(ip, max_requests) {
        Ok(()) => next.run(req).await,
        Err(resp) => resp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn allows_requests_under_limit() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        for _ in 0..5 {
            assert!(limiter.check(ip, 5).is_ok(), "should allow up to 5");
        }
    }

    #[test]
    fn blocks_request_over_limit() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..3 {
            assert!(limiter.check(ip, 3).is_ok());
        }
        assert!(
            limiter.check(ip, 3).is_err(),
            "4th request should be blocked"
        );
    }

    #[test]
    fn different_ips_have_independent_counters() {
        let limiter = RateLimiter::new();
        let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        assert!(limiter.check(ip_a, 2).is_ok());
        assert!(limiter.check(ip_a, 2).is_ok());
        assert!(limiter.check(ip_a, 2).is_err(), "IP A should be blocked");

        assert!(
            limiter.check(ip_b, 2).is_ok(),
            "IP B should still be allowed"
        );
        assert!(limiter.check(ip_b, 2).is_ok());
        assert!(
            limiter.check(ip_b, 2).is_err(),
            "IP B should also be blocked"
        );
    }

    #[test]
    fn api_key_tier_gets_higher_limit() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));

        // Default tier: 30 req/min
        for _ in 0..30 {
            assert!(limiter.check(ip, 30).is_ok());
        }
        assert!(
            limiter.check(ip, 30).is_err(),
            "31st default should be blocked"
        );

        // API key tier: should start fresh for new window (can't test in same window)
        // Just verify the higher limit is accepted
        let ip2 = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
        for _ in 0..300 {
            assert!(
                limiter.check(ip2, 300).is_ok(),
                "should allow 300 with API key"
            );
        }
        assert!(
            limiter.check(ip2, 300).is_err(),
            "301st API key should be blocked"
        );
    }
}
