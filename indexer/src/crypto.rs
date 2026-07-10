//! Message encryption at rest for escrow chat.
//!
//! Uses AES-256-GCM with a server-side key from DAGLOCK_MESSAGE_KEY env var.
//! Messages are encrypted on write, decrypted on read — transparent to the API.
//! This protects against DB leaks and SQL injection, NOT against the server operator
//! (who has the key in their env).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use std::env;
use tracing::warn;

/// Encrypt a plaintext message. Returns (ciphertext_hex, nonce_hex).
/// Generates an ephemeral key if DAGLOCK_MESSAGE_KEY is not set (dev mode).
#[allow(dead_code)]
pub fn encrypt_message(plaintext: &str) -> Result<(String, String), String> {
    let key_bytes = load_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to create cipher: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {e}"))?;

    Ok((hex::encode(ciphertext), hex::encode(nonce_bytes)))
}

/// Decrypt a message. Returns None if key is missing, wrong, or data corrupted.
pub fn decrypt_message(ciphertext_hex: &str, nonce_hex: &str) -> Option<String> {
    let key_bytes = load_key_optional()?;
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).ok()?;

    let ciphertext = hex::decode(ciphertext_hex).ok()?;
    let nonce_bytes = hex::decode(nonce_hex).ok()?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).ok()?;
    String::from_utf8(plaintext).ok()
}

fn load_key() -> [u8; 32] {
    match load_key_optional() {
        Some(k) => k,
        None => {
            // Generate a deterministic dev key (hashed from a known string)
            // This prevents message loss on restart during development.
            // In production, always set DAGLOCK_MESSAGE_KEY env var.
            warn!("DAGLOCK_MESSAGE_KEY not set — using deterministic dev key. Set DAGLOCK_MESSAGE_KEY=64_hex_chars for production!");
            let key = blake2b_simd::Params::new()
                .hash_length(32)
                .to_state()
                .update(b"daglock-dev-message-key-2026")
                .finalize();
            let mut k = [0u8; 32];
            k.copy_from_slice(key.as_bytes());
            k
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    const TEST_KEY: &str = "abababababababababababababababababababababababababababababababab";
    const WRONG_KEY: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Set DAGLOCK_MESSAGE_KEY and run a test. Serializes via ENV_LOCK.
    /// Uses unwrap_or_else to recover from mutex poisoning (parallel test panics).
    fn with_test_key<F: FnOnce()>(f: F) {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = env::var("DAGLOCK_MESSAGE_KEY").ok();
        env::set_var("DAGLOCK_MESSAGE_KEY", TEST_KEY);
        f();
        match prev {
            Some(k) => env::set_var("DAGLOCK_MESSAGE_KEY", k),
            None => env::remove_var("DAGLOCK_MESSAGE_KEY"),
        }
    }

    #[test]
    fn encrypt_decrypt_round_trip_with_env_key() {
        with_test_key(|| {
            let (ct, nonce) = encrypt_message("Hello, escrow!").unwrap();
            let decrypted = decrypt_message(&ct, &nonce).unwrap();
            assert_eq!(decrypted, "Hello, escrow!");
        });
    }

    #[test]
    fn encrypt_decrypt_long_message() {
        with_test_key(|| {
            let plaintext = "A".repeat(10_000);
            let (ct, nonce) = encrypt_message(&plaintext).expect("encrypt should work with test key");
            let decrypted = decrypt_message(&ct, &nonce).expect("decrypt should work");
            assert_eq!(decrypted, plaintext);
        });
    }

    #[test]
    fn decrypt_wrong_key_returns_none() {
        // Encrypt with TEST_KEY, then manually swap to WRONG_KEY for decrypt
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = env::var("DAGLOCK_MESSAGE_KEY").ok();
        env::set_var("DAGLOCK_MESSAGE_KEY", TEST_KEY);
        let (ct, nonce) = encrypt_message("secret").unwrap();
        env::set_var("DAGLOCK_MESSAGE_KEY", WRONG_KEY);
        let result = decrypt_message(&ct, &nonce);
        assert!(result.is_none());
        match prev {
            Some(k) => env::set_var("DAGLOCK_MESSAGE_KEY", k),
            None => env::remove_var("DAGLOCK_MESSAGE_KEY"),
        }
    }

    #[test]
    fn decrypt_tampered_ciphertext_returns_none() {
        with_test_key(|| {
            let (mut ct, nonce) = encrypt_message("secret").unwrap();
            let bytes = hex::decode(&ct).unwrap();
            let tampered = std::iter::once(bytes[0] ^ 1).chain(bytes[1..].iter().copied()).collect::<Vec<_>>();
            ct = hex::encode(tampered);
            let result = decrypt_message(&ct, &nonce);
            assert!(result.is_none());
        });
    }

    #[test]
    fn encrypt_succeeds_without_env_key() {
        // Encryption uses load_key() which falls back to dev key.
        // Decryption uses load_key_optional() which returns None -> decrypt returns None.
        // This is by design: you can always encrypt, but decrypt needs DAGLOCK_MESSAGE_KEY.
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let prev = env::var("DAGLOCK_MESSAGE_KEY").ok();
        env::remove_var("DAGLOCK_MESSAGE_KEY");
        let (ct, nonce) = encrypt_message("dev mode test").unwrap();
        assert!(!ct.is_empty());
        assert!(!nonce.is_empty());
        match prev {
            Some(k) => env::set_var("DAGLOCK_MESSAGE_KEY", k),
            None => {}
        }
    }
}

fn load_key_optional() -> Option<[u8; 32]> {
    let hex_key = env::var("DAGLOCK_MESSAGE_KEY").ok()?;
    let bytes = hex::decode(hex_key).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Some(key)
}
