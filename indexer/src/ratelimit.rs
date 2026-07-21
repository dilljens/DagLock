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
            Self::Free => 60,
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
/// Maximum number of tracked IP windows to prevent unbounded memory growth.
/// At ~72 bytes per entry, this caps at ~14 MB for the window map.
const MAX_WINDOW_ENTRIES: usize = 200_000;

pub struct RateLimiter {
    inner: Mutex<RateLimiterInner>,
}

struct RateLimiterInner {
    windows: HashMap<IpAddr, WindowState>,
    /// key_hash → (tier, cached_at) with 60s TTL
    tier_cache: HashMap<Vec<u8>, (ApiTier, Instant)>,
    /// Insertion order for LRU eviction when map exceeds MAX_WINDOW_ENTRIES
    insertion_order: std::collections::VecDeque<IpAddr>,
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
                insertion_order: std::collections::VecDeque::new(),
            }),
        }
    }

    /// Remove expired windows and stale tier cache entries.
    /// Call periodically (e.g. every 5 minutes) to prevent unbounded memory growth.
    pub fn cleanup(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        // Remove expired windows (entries older than 2 windows)
        let cutoff = now - Duration::from_secs(WINDOW_SECS * 2);
        inner.windows.retain(|_, w| w.reset_at > cutoff);

        // Remove stale tier cache entries (older than 5 minutes)
        let tier_cutoff = now - Duration::from_secs(300);
        inner.tier_cache.retain(|_, (_tier, cached_at)| *cached_at > tier_cutoff);
    }

    /// Check rate limit for an IP with an optional tier override.
    /// `tier` is `None` when no API key is present (falls back to Free).
    pub fn check(&self, ip: IpAddr, tier: Option<ApiTier>) -> Result<(), Response> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        let max_requests = tier.unwrap_or(ApiTier::Free).max_requests();

        // Promote IP to back of insertion order (recently active) — before mutable borrow
        if inner.windows.contains_key(&ip) {
            if let Some(pos) = inner.insertion_order.iter().position(|x| *x == ip) {
                inner.insertion_order.remove(pos);
                inner.insertion_order.push_back(ip);
            }
        }

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
                // Include CORS headers so the browser shows the actual error, not a CORS error
                let mut resp = (
                    axum::http::StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({
                        "error": "rate_limited",
                        "message": format!(
                            "Rate limit exceeded. Max {} requests per {} seconds.",
                            entry.max_requests, WINDOW_SECS
                        ),
                    })),
                )
                    .into_response();
                resp.headers_mut().insert(
                    axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    axum::http::HeaderValue::from_static("*"),
                );
                Err(resp)
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
            inner.insertion_order.push_back(ip);

            // Evict oldest entries if over capacity
            while inner.windows.len() > MAX_WINDOW_ENTRIES {
                if let Some(oldest) = inner.insertion_order.pop_front() {
                    if inner.windows.get(&oldest).map_or(false, |w| w.reset_at <= now) {
                        inner.windows.remove(&oldest);
                    }
                } else {
                    break;
                }
            }

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
    // Resolve the real client IP:
    // 1. Use the actual TCP connection IP (most trustworthy)
    // 2. Fall back to X-Forwarded-For's RIGHTMOST IP (the one added by the outermost
    //    trusted proxy in a well-configured reverse proxy chain — leftmost is
    //    client-supplied and can be spoofed)
    let ip = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.ip())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                // Take the RIGHTMOST IP (trusted proxy), not the leftmost (client-spoofable)
                .and_then(|v| v.rsplit(',').next())
                .and_then(|v| v.trim().parse::<IpAddr>().ok())
        })
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
    fn free_tier_allows_60() {
        let limiter = RateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        for _ in 0..60 {
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
        for _ in 0..60 {
            assert!(limiter.check(ip, None).is_ok());
        }
        assert!(limiter.check(ip, None).is_err());
    }

    #[test]
    fn different_ips_have_independent_counters() {
        let limiter = RateLimiter::new();
        let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
        let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

        for _ in 0..60 {
            assert!(limiter.check(ip_a, Some(ApiTier::Free)).is_ok());
        }
        assert!(limiter.check(ip_a, Some(ApiTier::Free)).is_err());

        // IP B starts fresh
        for _ in 0..60 {
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
