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

#[allow(dead_code)]
pub async fn count_messages(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM escrow_messages WHERE escrow_id = ?1")
            .bind(escrow_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

/// Get the chat pubkey for a party in an escrow.
/// Returns `None` if the address is not a party or no pubkey was registered.
pub async fn get_chat_pubkey(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    address: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT chat_pubkey_buyer, chat_pubkey_seller FROM escrows WHERE id = ?1"
    )
    .bind(escrow_id)
    .fetch_optional(pool)
    .await?;
    match row {
        Some(r) => {
            let buyer_pubkey: Option<String> = r.try_get("chat_pubkey_buyer").ok().flatten();
            let seller_pubkey: Option<String> = r.try_get("chat_pubkey_seller").ok().flatten();
            // Check if address is buyer or seller and return the matching pubkey
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
