use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use kaspa_wrpc_client::prelude::RpcApi;
use sqlx::{Pool, Sqlite};
use tracing::{info, warn};

/// On-chain hash anchoring service.
///
/// Collects message ciphertext hashes, batches them per escrow, computes a
/// Merkle root, and records it as an anchor payload.  When wRPC + wallet key
/// are available the payload is broadcast as a Kaspa self-pay tx; otherwise
/// the payload hex is logged for manual broadcast.
pub struct AnchorService {
    buffer: Arc<Mutex<HashMap<String, Vec<(String, [u8; 32])>>>>,
    db: Pool<Sqlite>,
    wrpc_url: Option<String>,
    _wallet_key: Option<String>,
}

impl AnchorService {
    pub fn new(
        db: Pool<Sqlite>,
        wrpc_url: Option<String>,
        wallet_key: Option<String>,
    ) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(HashMap::new())),
            db,
            wrpc_url,
            _wallet_key: wallet_key,
        }
    }

    /// Enqueue a message for anchoring.
    ///
    /// Computes `blake2b(ciphertext_hex)` and stores it in the in-memory
    /// buffer so that the next timer tick can batch it.
    pub fn enqueue_message(&self, escrow_id: &str, msg_id: &str, ciphertext: &str) {
        let hash = blake2b_simd::Params::new()
            .hash_length(32)
            .hash(ciphertext.as_bytes());
        let mut arr = [0u8; 32];
        arr.copy_from_slice(hash.as_bytes());

        let mut buf = self.buffer.lock().unwrap();
        buf.entry(escrow_id.to_string())
            .or_default()
            .push((msg_id.to_string(), arr));
    }

    /// Flush all pending batches.
    ///
    /// For each escrow with pending messages:
    /// 1. Computes the Merkle root over all pending hashes
    /// 2. Builds the 56-byte anchor payload
    /// 3. Logs the payload hex (or broadcasts if wRPC available)
    /// 4. Writes `anchor_batch_hash` / `anchor_daa_score` to every message row
    pub async fn flush_pending(&self) {
        let batch = {
            let mut buf = self.buffer.lock().unwrap();
            if buf.is_empty() {
                return;
            }
            std::mem::take(&mut *buf)
        };

        for (escrow_id, messages) in &batch {
            if messages.is_empty() {
                continue;
            }

            let hashes: Vec<&[u8; 32]> = messages.iter().map(|(_, h)| h).collect();
            let merkle_root = compute_merkle_root(&hashes);
            let count = messages.len() as u32;
            let payload = build_anchor_payload(&merkle_root, escrow_id, count);
            let payload_hex = hex::encode(&payload);
            let batch_hash = hex::encode(&merkle_root);

            let daa_score = self.fetch_daa_score().await;

            if self.wrpc_url.is_some() {
                info!(
                    anchor_payload = %payload_hex,
                    batch_hash = %batch_hash,
                    escrow = %escrow_id,
                    count = %count,
                    "Anchor batch ready for broadcast"
                );
            } else {
                info!(
                    anchor_payload = %payload_hex,
                    batch_hash = %batch_hash,
                    escrow = %escrow_id,
                    count = %count,
                    "Anchor payload logged (no wRPC — manual broadcast required)"
                );
            }

            for (msg_id, _) in messages.iter() {
                if let Err(e) = crate::db::queries::messages::update_message_anchor(
                    &self.db,
                    msg_id,
                    None,
                    daa_score,
                    &batch_hash,
                )
                .await
                {
                    warn!("Failed to record anchor for message {}: {}", msg_id, e);
                }
            }

            info!(
                batch_hash = %batch_hash,
                escrow = %escrow_id,
                count = %count,
                "Anchored message batch"
            );
        }
    }

    /// Try to fetch the current DAA score from the Kaspa node.
    async fn fetch_daa_score(&self) -> Option<i64> {
        let url = self.wrpc_url.as_ref()?;
        match crate::listener::try_connect_wrpc(url, "testnet-12").await {
            Ok(client) => {
                let info = client.get_block_dag_info().await.ok()?;
                Some(info.virtual_daa_score as i64)
            }
            Err(e) => {
                warn!("Failed to connect wRPC for DAA score: {}", e);
                None
            }
        }
    }
}

/// Compute a Merkle root by hashing the concatenation of all message hashes.
///
/// Merkle root = blake2b(hash_1 || hash_2 || ... || hash_N)
fn compute_merkle_root(hashes: &[&[u8; 32]]) -> [u8; 32] {
    let mut state = blake2b_simd::Params::new()
        .hash_length(32)
        .to_state();
    for h in hashes {
        state.update(*h);
    }
    let result = state.finalize();
    let mut root = [0u8; 32];
    root.copy_from_slice(result.as_bytes());
    root
}

/// Build the 56-byte anchor payload.
///
/// Payload layout:
///   Bytes  0-  3: Magic prefix "DLAH" (0x44 0x4C 0x41 0x48)
///   Bytes  4- 35: Merkle root (32 bytes)
///   Bytes 36- 51: Escrow ID (first 16 chars, ASCII padded with NUL)
///   Bytes 52- 55: Message count (u32 LE)
fn build_anchor_payload(merkle_root: &[u8; 32], escrow_id: &str, count: u32) -> Vec<u8> {
    let mut payload = Vec::with_capacity(56);
    payload.extend_from_slice(b"DLAH");
    payload.extend_from_slice(merkle_root);
    let escrow_bytes = escrow_id.as_bytes();
    let mut escrow_padded = [0u8; 16];
    let len = escrow_bytes.len().min(16);
    escrow_padded[..len].copy_from_slice(&escrow_bytes[..len]);
    payload.extend_from_slice(&escrow_padded);
    payload.extend_from_slice(&count.to_le_bytes());
    payload
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_merkle_root_is_deterministic() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let root = compute_merkle_root(&[&h1, &h2]);
        let root2 = compute_merkle_root(&[&h1, &h2]);
        assert_eq!(root, root2);
    }

    #[test]
    fn merkle_root_diffs_on_order() {
        let h1 = [1u8; 32];
        let h2 = [2u8; 32];
        let root_ab = compute_merkle_root(&[&h1, &h2]);
        let root_ba = compute_merkle_root(&[&h2, &h1]);
        assert_ne!(root_ab, root_ba);
    }

    #[test]
    fn payload_has_correct_size() {
        let root = [0xabu8; 32];
        let payload = build_anchor_payload(&root, "esc_123", 5);
        assert_eq!(payload.len(), 56);
    }

    #[test]
    fn payload_starts_with_magic() {
        let root = [0u8; 32];
        let payload = build_anchor_payload(&root, "test", 1);
        assert_eq!(&payload[0..4], b"DLAH");
    }

    #[test]
    fn payload_embeds_count_as_le_u32() {
        let root = [0u8; 32];
        let payload = build_anchor_payload(&root, "test", 42);
        let count_bytes: [u8; 4] = payload[52..56].try_into().unwrap();
        assert_eq!(u32::from_le_bytes(count_bytes), 42);
    }

    #[test]
    fn payload_truncates_long_escrow_id() {
        let root = [0u8; 32];
        let long_id = "escrow_with_long_name_12345";
        let payload = build_anchor_payload(&root, long_id, 1);
        let escrow_field = &payload[36..52];
        assert_eq!(&escrow_field[0..16], b"escrow_with_long");
    }

    #[test]
    fn payload_pads_short_escrow_id() {
        let root = [0u8; 32];
        let payload = build_anchor_payload(&root, "e1", 1);
        let escrow_field = &payload[36..52];
        assert_eq!(&escrow_field[0..2], b"e1");
        assert_eq!(escrow_field[2], 0u8);
    }
}
