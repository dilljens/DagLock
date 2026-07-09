//! Tiered rate limiter using axum::middleware::from_fn.
//!
//! API key tier determines the request limit per window:
//!   free  → 10 req/min
//!   pro   → 100 req/min
//!   whale → 1000 req/min
//!
//! A simple in-memory cache (TTL 60s) avoids a DB lookup on every request.

use axum::{
    http::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Mutex,
    time::{Duration, Instant},
};

const WINDOW_SECS: u64 = 60;

/// Supported API tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiTier {
    Free,
    Pro,
    Whale,
}

impl ApiTier {
    fn max_requests(self) -> u32 {
        match self {
            Self::Free => 10,
            Self::Pro => 100,
            Self::Whale => 1000,
        }
    }
}

impl From<&str> for ApiTier {
    fn from(s: &str) -> Self {
        match s {
            "pro" => Self::Pro,
            "whale" => Self::Whale,
            _ => Self::Free,
        }
    }
}

/// Thread-safe per-IP rate limiter with tier support.
pub struct RateLimiter {
    inner: Mutex<RateLimiterInner>,
}

struct RateLimiterInner {
    windows: HashMap<IpAddr, WindowState>,
    /// key_hash → (tier, cached_at) with 60s TTL
    tier_cache: HashMap<Vec<u8>, (ApiTier, Instant)>,
}

struct WindowState {
    count: u32,
    reset_at: Instant,
    max_requests: u32,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(RateLimiterInner {
                windows: HashMap::new(),
                tier_cache: HashMap::new(),
            }),
        }
    }

    /// Check rate limit for an IP with an optional tier override.
    /// `tier` is `None` when no API key is present (falls back to Free).
    pub fn check(&self, ip: IpAddr, tier: Option<ApiTier>) -> Result<(), Response> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        let max_requests = tier.unwrap_or(ApiTier::Free).max_requests();

        if let Some(entry) = inner.windows.get_mut(&ip) {
            if now >= entry.reset_at {
                entry.count = 1;
                entry.reset_at = now + Duration::from_secs(WINDOW_SECS);
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
            inner.windows.insert(
                ip,
                WindowState {
                    count: 1,
                    reset_at: now + Duration::from_secs(WINDOW_SECS),
                    max_requests,
                },
            );
            Ok(())
        }
    }

    /// Resolve an API key hash to its tier, using a 60s cache.
    /// `None` means no valid key was provided.
    pub fn resolve_tier(&self, key_hash: Option<&[u8]>) -> Option<ApiTier> {
        let key_hash = key_hash?;
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        // Check cache
        if let Some((tier, cached_at)) = inner.tier_cache.get(key_hash) {
            if now.duration_since(*cached_at) < Duration::from_secs(60) {
                return Some(*tier);
            }
        }

        // Cache miss — caller must populate via cache_tier()
        None
    }

    /// Populate the tier cache for a key hash.
    pub fn cache_tier(&self, key_hash: Vec<u8>, tier: ApiTier) {
        if let Ok(mut inner) = self.inner.lock() {
            inner
                .tier_cache
                .insert(key_hash, (tier, Instant::now()));
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Axum middleware for tiered rate limiting.
///
/// Headers inspected:
///   X-Daglock-Api-Key — hashed to look up the key's tier
///
/// When no (or invalid) API key is present, the Free tier applies.
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

    let api_key = req
        .headers()
        .get("x-daglock-api-key")
        .and_then(|v| v.to_str().ok())
        .filter(|v| !v.is_empty());

    let tier = api_key.and_then(|raw| {
        let hash = blake2b_simd::Params::new()
            .hash_length(32)
            .hash(raw.as_bytes())
            .as_bytes()
            .to_vec();

        // Try cache first; if miss, we default to Free rather than
        // blocking the request on a DB query. The cache is populated
        // after successful key verification by endpoint handlers.
        state.resolve_tier(Some(&hash)).or_else(|| {
            // Fall back to Free for uncached keys
            // (will be cached after the first verified request)
            Some(ApiTier::Free)
        })
    });

    match state.check(ip, tier) {
        Ok(()) => next.run(req).await,
        Err(resp) => resp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn free_tier_allows_10() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..10 {
            assert!(limiter.check(ip, Some(ApiTier::Free)).is_ok());
        }
        assert!(limiter.check(ip, Some(ApiTier::Free)).is_err());
    }

    #[test]
    fn pro_tier_allows_100() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        for _ in 0..100 {
            assert!(limiter.check(ip, Some(ApiTier::Pro)).is_ok());
        }
        assert!(limiter.check(ip, Some(ApiTier::Pro)).is_err());
    }

    #[test]
    fn whale_tier_allows_1000() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3));
        for _ in 0..1000 {
            assert!(limiter.check(ip, Some(ApiTier::Whale)).is_ok());
        }
        assert!(limiter.check(ip, Some(ApiTier::Whale)).is_err());
    }

    #[test]
    fn no_tier_falls_back_to_free() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4));
        for _ in 0..10 {
            assert!(limiter.check(ip, None).is_ok());
        }
        assert!(limiter.check(ip, None).is_err());
    }

    #[test]
    fn different_ips_have_independent_counters() {
        let limiter = RateLimiter::new();
        let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        for _ in 0..10 {
            assert!(limiter.check(ip_a, Some(ApiTier::Free)).is_ok());
        }
        assert!(limiter.check(ip_a, Some(ApiTier::Free)).is_err());

        // IP B starts fresh
        for _ in 0..10 {
            assert!(limiter.check(ip_b, Some(ApiTier::Free)).is_ok());
        }
    }

    #[test]
    fn tier_cache_roundtrip() {
        let limiter = RateLimiter::new();
        let hash = vec![1u8, 2, 3];
        assert!(limiter.resolve_tier(Some(&hash)).is_none());
        limiter.cache_tier(hash.clone(), ApiTier::Pro);
        assert_eq!(
            limiter.resolve_tier(Some(&hash)),
            Some(ApiTier::Pro)
        );
    }
}
