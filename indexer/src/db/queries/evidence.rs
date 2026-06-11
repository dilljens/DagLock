use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_evidence(
    pool: &Pool<Sqlite>,
    evidence: &DisputeEvidence,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dispute_evidence (id, escrow_id, submitted_by, content, content_hash, signed_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&evidence.id)
    .bind(&evidence.escrow_id)
    .bind(&evidence.submitted_by)
    .bind(&evidence.content)
    .bind(&evidence.content_hash)
    .bind(&evidence.signed_message)
    .bind(evidence.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_evidence(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<DisputeEvidence>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT * FROM dispute_evidence WHERE escrow_id = ?1 ORDER BY created_at ASC")
            .bind(escrow_id)
            .fetch_all(pool)
            .await?;
    let evidence = rows
        .into_iter()
        .map(|row| DisputeEvidence {
            id: row.try_get("app_id_field").unwrap_or_default(),
            escrow_id: row.try_get("escrow_id").unwrap_or_default(),
            submitted_by: row.try_get("submitted_by").unwrap_or_default(),
            content: row.try_get("content").unwrap_or_default(),
            content_hash: row.try_get("content_hash").unwrap_or_default(),
            signed_message: row.try_get("signed_message").unwrap_or(None),
            created_at: row.try_get("created_at").unwrap_or(0),
        })
        .collect();
    Ok(evidence)
}

pub async fn resolve_dispute(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    outcome: &str,
    _resolved_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE escrows SET dispute_outcome = ?1, dispute_resolved_at = ?2 WHERE id = ?3 AND status = 'disputed'",
    )
    .bind(outcome)
    .bind(chrono::Utc::now().timestamp())
    .bind(escrow_id)
    .execute(pool)
    .await?;
    Ok(())
}
