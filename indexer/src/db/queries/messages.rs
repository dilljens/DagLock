use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_message(
    pool: &Pool<Sqlite>,
    msg: &EscrowMessage,
    content_enc: &str,
    nonce: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO escrow_messages (id, escrow_id, sender_address, content_enc, nonce, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )
    .bind(&msg.id)
    .bind(&msg.escrow_id)
    .bind(&msg.sender_address)
    .bind(content_enc)
    .bind(nonce)
    .bind(msg.created_at)
    .execute(pool).await?;
    Ok(())
}

pub async fn list_messages_raw(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<(String, String, String, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT sender_address, content_enc, nonce, created_at FROM escrow_messages WHERE escrow_id = ?1 ORDER BY created_at ASC"
    )
    .bind(escrow_id)
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.try_get::<String, _>("sender_address").unwrap_or_default(),
                r.try_get::<String, _>("content_enc").unwrap_or_default(),
                r.try_get::<String, _>("nonce").unwrap_or_default(),
                r.try_get::<i64, _>("created_at").unwrap_or(0),
            )
        })
        .collect())
}

pub async fn list_messages_with_anchors(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<AnchoredMessage>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, sender_address, content_enc, nonce, created_at, anchor_tx_id, anchor_daa_score, anchor_batch_hash
         FROM escrow_messages WHERE escrow_id = ?1 ORDER BY created_at ASC"
    )
    .bind(escrow_id)
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            AnchoredMessage {
                id: r.try_get::<String, _>("id").unwrap_or_default(),
                sender_address: r.try_get::<String, _>("sender_address").unwrap_or_default(),
                content_enc: r.try_get::<String, _>("content_enc").unwrap_or_default(),
                nonce: r.try_get::<String, _>("nonce").unwrap_or_default(),
                created_at: r.try_get::<i64, _>("created_at").unwrap_or(0),
                anchor_tx_id: r.try_get("anchor_tx_id").ok().flatten(),
                anchor_daa_score: r.try_get("anchor_daa_score").ok().flatten(),
                anchor_batch_hash: r.try_get("anchor_batch_hash").ok().flatten(),
            }
        })
        .collect())
}

pub async fn update_message_anchor(
    pool: &Pool<Sqlite>,
    msg_id: &str,
    anchor_tx_id: Option<&str>,
    anchor_daa_score: Option<i64>,
    anchor_batch_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE escrow_messages SET anchor_tx_id = ?1, anchor_daa_score = ?2, anchor_batch_hash = ?3 WHERE id = ?4"
    )
    .bind(anchor_tx_id)
    .bind(anchor_daa_score)
    .bind(anchor_batch_hash)
    .bind(msg_id)
    .execute(pool).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn count_messages(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM escrow_messages WHERE escrow_id = ?1")
            .bind(escrow_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn count_unanchored_messages(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM escrow_messages WHERE escrow_id = ?1 AND anchor_batch_hash IS NULL")
            .bind(escrow_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

pub async fn get_anchor_summary(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<AnchorBatch>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT anchor_batch_hash, anchor_tx_id, anchor_daa_score,
                COUNT(*) as message_count,
                MIN(created_at) as from_time, MAX(created_at) as to_time
         FROM escrow_messages
         WHERE escrow_id = ?1 AND anchor_batch_hash IS NOT NULL
         GROUP BY anchor_batch_hash
         ORDER BY MIN(created_at) ASC"
    )
    .bind(escrow_id)
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            AnchorBatch {
                batch_hash: r.try_get::<String, _>("anchor_batch_hash").unwrap_or_default(),
                anchor_tx_id: r.try_get("anchor_tx_id").ok().flatten(),
                anchor_daa_score: r.try_get("anchor_daa_score").ok().flatten(),
                message_count: r.try_get::<i64, _>("message_count").unwrap_or(0),
                from_time: r.try_get::<i64, _>("from_time").unwrap_or(0),
                to_time: r.try_get::<i64, _>("to_time").unwrap_or(0),
            }
        })
        .collect())
}

/// Update the chat pubkey for a party in an escrow.
/// Determines which column to update based on the caller's address.
pub async fn update_chat_pubkey(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    address: &str,
    pubkey: &str,
) -> Result<(), sqlx::Error> {
    // First determine if the caller is buyer or seller
    let row = sqlx::query("SELECT buyer_address, seller_address FROM escrows WHERE id = ?1")
        .bind(escrow_id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => {
            let buyer_addr: String = r.try_get("buyer_address").unwrap_or_default();
            let seller_addr: Option<String> = r.try_get("seller_address").ok().flatten();
            if address == buyer_addr {
                sqlx::query("UPDATE escrows SET chat_pubkey_buyer = ?1 WHERE id = ?2")
                    .bind(pubkey)
                    .bind(escrow_id)
                    .execute(pool)
                    .await?;
            } else if seller_addr.as_deref() == Some(address) {
                sqlx::query("UPDATE escrows SET chat_pubkey_seller = ?1 WHERE id = ?2")
                    .bind(pubkey)
                    .bind(escrow_id)
                    .execute(pool)
                    .await?;
            }
            Ok(())
        }
        None => Ok(()),
    }
}

/// Get the chat pubkey for a party in an escrow.
/// Returns `None` if the address is not a party or no pubkey was registered.
pub async fn get_chat_pubkey(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    address: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT chat_pubkey_buyer, chat_pubkey_seller, buyer_address, seller_address FROM escrows WHERE id = ?1"
    )
    .bind(escrow_id)
    .fetch_optional(pool)
    .await?;
    match row {
        Some(r) => {
            let buyer_pubkey: Option<String> = r.try_get("chat_pubkey_buyer").ok().flatten();
            let seller_pubkey: Option<String> = r.try_get("chat_pubkey_seller").ok().flatten();
            let buyer_addr: String = r.try_get("buyer_address").unwrap_or_default();
            let seller_addr: Option<String> = r.try_get("seller_address").ok().flatten();
            if address == buyer_addr {
                Ok(buyer_pubkey)
            } else if seller_addr.as_deref() == Some(address) {
                Ok(seller_pubkey)
            } else {
                Ok(None)
            }
        }
        None => Ok(None),
    }
}
