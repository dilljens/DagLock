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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn allows_requests_under_limit() {
        let limiter = RateLimiter::new(5, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
        for _ in 0..5 {
            assert!(limiter.check(ip).is_ok(), "should allow up to 5");
        }
    }

    #[test]
    fn blocks_request_over_limit() {
        let limiter = RateLimiter::new(3, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..3 {
            assert!(limiter.check(ip).is_ok());
        }
        assert!(limiter.check(ip).is_err(), "4th request should be blocked");
    }

    #[test]
    fn different_ips_have_independent_counters() {
        let limiter = RateLimiter::new(2, 60);
        let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        assert!(limiter.check(ip_a).is_ok());
        assert!(limiter.check(ip_a).is_ok());
        assert!(limiter.check(ip_a).is_err(), "IP A should be blocked");

        assert!(limiter.check(ip_b).is_ok(), "IP B should still be allowed");
        assert!(limiter.check(ip_b).is_ok());
        assert!(limiter.check(ip_b).is_err(), "IP B should also be blocked");
    }
}
