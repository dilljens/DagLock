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
pub fn encrypt_message(plaintext: &str) -> (String, String) {
    let key_bytes = load_key();
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).expect("valid AES-256 key");

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .expect("encryption should succeed");

    (hex::encode(ciphertext), hex::encode(nonce_bytes))
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
            warn!("DAGLOCK_MESSAGE_KEY not set — generating ephemeral key. Messages lost on restart!");
            let mut key = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut key);
            key
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
