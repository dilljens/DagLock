//! Atomic swap API endpoints.

use axum::Json;
use serde_json::{json, Value};

/// POST /v1/swap/generate
/// Generate a random secret and compute its SHA-256 hash for atomic swaps.
///
/// The secret is returned to the caller ONCE and immediately zeroized
/// from server memory — it is NOT persisted server-side.
pub async fn generate() -> Json<Value> {
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    let mut secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret);

    let mut hasher = Sha256::new();
    hasher.update(secret);
    let hash = hasher.finalize();

    let secret_hex = hex::encode(secret);

    // Zeroize the secret from server memory — it is NOT stored server-side.
    // The caller must save it themselves.
    secret.fill(0);

    Json(json!({
        "secret": secret_hex,
        "hash": hex::encode(hash),
    }))
}
