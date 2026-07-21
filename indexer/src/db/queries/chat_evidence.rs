use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn reveal_chat_key(
    pool: &Pool<Sqlite>,
    case_id: &str,
    encrypted_key: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE jury_cases SET revealed_chat_key_enc = ?1, revealed_at = ?2 WHERE id = ?3")
        .bind(encrypted_key)
        .bind(now)
        .bind(case_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_revealed_chat_key(
    pool: &Pool<Sqlite>,
    case_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT revealed_chat_key_enc FROM jury_cases WHERE id = ?1 AND revealed_chat_key_enc IS NOT NULL",
    )
    .bind(case_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|r| r.try_get::<String, _>("revealed_chat_key_enc").ok()))
}

pub async fn store_decrypted_evidence(
    pool: &Pool<Sqlite>,
    case_id: &str,
    messages: &[EvidenceMessage],
) -> Result<(), sqlx::Error> {
    for msg in messages {
        sqlx::query(
            "INSERT OR REPLACE INTO mediation_evidence (id, case_id, message_id, sender_address, decrypted_content, created_at, anchor_tx_id, anchor_daa_score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )
        .bind(&msg.id)
        .bind(case_id)
        .bind(&msg.id)
        .bind(&msg.sender_address)
        .bind(&msg.decrypted_content)
        .bind(msg.created_at)
        .bind(&msg.anchor_tx_id)
        .bind(msg.anchor_daa_score)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_decrypted_evidence(
    pool: &Pool<Sqlite>,
    case_id: &str,
) -> Result<Vec<EvidenceMessage>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, sender_address, decrypted_content, created_at, anchor_tx_id, anchor_daa_score
         FROM mediation_evidence WHERE case_id = ?1 ORDER BY created_at ASC",
    )
    .bind(case_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| EvidenceMessage {
            id: r.try_get::<String, _>("id").unwrap_or_default(),
            sender_address: r.try_get::<String, _>("sender_address").unwrap_or_default(),
            decrypted_content: r
                .try_get::<String, _>("decrypted_content")
                .unwrap_or_default(),
            created_at: r.try_get::<i64, _>("created_at").unwrap_or(0),
            anchor_tx_id: r.try_get("anchor_tx_id").ok().flatten(),
            anchor_daa_score: r.try_get("anchor_daa_score").ok().flatten(),
        })
        .collect())
}

pub async fn clear_evidence(pool: &Pool<Sqlite>, case_id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("DELETE FROM mediation_evidence WHERE case_id = ?1")
        .bind(case_id)
        .execute(pool)
        .await?;
    sqlx::query(
        "UPDATE jury_cases SET revealed_chat_key_enc = NULL, evidence_cleared_at = ?1 WHERE id = ?2",
    )
    .bind(now)
    .bind(case_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_active_reveals(pool: &Pool<Sqlite>) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, revealed_at FROM jury_cases
         WHERE revealed_chat_key_enc IS NOT NULL AND revealed_at IS NOT NULL
         AND evidence_cleared_at IS NULL",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let id: String = r.try_get("id").ok()?;
            let revealed_at: i64 = r.try_get("revealed_at").ok()?;
            Some((id, revealed_at))
        })
        .collect())
}
