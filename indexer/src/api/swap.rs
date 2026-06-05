//! Atomic swap API endpoints.

use axum::Json;
use serde_json::{json, Value};

/// POST /v1/swap/generate
/// Generate a random secret and compute its SHA-256 hash for atomic swaps.
pub async fn generate() -> Json<Value> {
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);

    let mut hasher = Sha256::new();
    hasher.update(secret);
    let hash = hasher.finalize();

    Json(json!({
        "secret": hex::encode(secret),
        "hash": hex::encode(hash),
    }))
}
