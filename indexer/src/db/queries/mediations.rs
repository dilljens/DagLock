use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn initiate_mediation(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    buyer_claim: &str,
    seller_claim: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let expires_at = now + 86_400; // 24 hours
    sqlx::query(
        "UPDATE escrows SET
            mediation_status = 'pending',
            mediation_buyer_claim = ?1,
            mediation_seller_claim = ?2,
            mediation_result = NULL,
            mediation_expires_at = ?3,
            mediation_buyer_accepted = 0,
            mediation_seller_accepted = 0
         WHERE id = ?4",
    )
    .bind(buyer_claim)
    .bind(seller_claim)
    .bind(expires_at)
    .bind(escrow_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn store_mediation_result(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    result: &MediationResult,
) -> Result<(), sqlx::Error> {
    let json = serde_json::to_string(result).unwrap_or_default();
    sqlx::query(
        "UPDATE escrows SET mediation_status = 'completed', mediation_result = ?1 WHERE id = ?2",
    )
    .bind(&json)
    .bind(escrow_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn accept_mediation(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    party: &str,
) -> Result<bool, sqlx::Error> {
    let column = match party {
        "buyer" => "mediation_buyer_accepted",
        "seller" => "mediation_seller_accepted",
        _ => return Ok(false),
    };
    let sql = format!("UPDATE escrows SET {column} = 1 WHERE id = ?1 AND {column} = 0");
    let result = sqlx::query(&sql).bind(escrow_id).execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn check_mediation_both_accepted(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query(
        "SELECT mediation_buyer_accepted, mediation_seller_accepted
         FROM escrows WHERE id = ?1",
    )
    .bind(escrow_id)
    .fetch_one(pool)
    .await?;
    let buyer: i64 = row.try_get("mediation_buyer_accepted").unwrap_or(0);
    let seller: i64 = row.try_get("mediation_seller_accepted").unwrap_or(0);
    Ok(buyer != 0 && seller != 0)
}

pub async fn get_mediation_status(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Option<(String, Option<String>, Option<i64>)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT mediation_status, mediation_result, mediation_expires_at
         FROM escrows WHERE id = ?1 AND mediation_status IS NOT NULL",
    )
    .bind(escrow_id)
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok(None);
    }
    let row = &rows[0];
    let status: String = row.try_get("mediation_status").unwrap_or_default();
    let result: Option<String> = row.try_get("mediation_result").ok().flatten();
    let expires_at: Option<i64> = row.try_get("mediation_expires_at").ok().flatten();
    Ok(Some((status, result, expires_at)))
}

pub async fn mark_mediation_escalated(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE escrows SET mediation_status = 'escalated' WHERE id = ?1")
        .bind(escrow_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn find_expired_mediations(
    pool: &Pool<Sqlite>,
    now: i64,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, amount_sompi FROM escrows
         WHERE mediation_status = 'completed'
           AND mediation_expires_at IS NOT NULL
           AND mediation_expires_at <= ?1
           AND (mediation_buyer_accepted = 0 OR mediation_seller_accepted = 0)",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let id: String = row.try_get("id").ok()?;
            let amount: i64 = row.try_get("amount_sompi").ok()?;
            Some((id, amount))
        })
        .collect())
}
