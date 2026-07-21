use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_multi_escrow(pool: &Pool<Sqlite>, m: &MultiEscrow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO multi_escrows (id, lock_tx_id, parties, shares, total_amount, status, created_at, settled_at, refunded_at, signatures)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&m.id)
    .bind(&m.lock_tx_id)
    .bind(serde_json::to_string(&m.parties).unwrap_or_default())
    .bind(serde_json::to_string(&m.shares).unwrap_or_default())
    .bind(m.total_amount)
    .bind(&m.status)
    .bind(m.created_at)
    .bind(m.settled_at)
    .bind(m.refunded_at)
    .bind(serde_json::to_string(&m.signatures).unwrap_or_default())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_multi_escrow(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<MultiEscrow>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM multi_escrows WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(row_to_multi))
}

pub async fn list_multi_by_address(
    pool: &Pool<Sqlite>,
    address: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MultiEscrow>, i64), sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM multi_escrows WHERE parties LIKE ?1")
        .bind(format!("%{}%", address))
        .fetch_one(pool)
        .await?;

    let rows = sqlx::query(
        "SELECT * FROM multi_escrows WHERE parties LIKE ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
    )
    .bind(format!("%{}%", address))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let escrows: Vec<MultiEscrow> = rows.into_iter().map(row_to_multi).collect();
    Ok((escrows, total.0))
}

pub async fn record_signature(
    pool: &Pool<Sqlite>,
    id: &str,
    address: &str,
) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT signatures FROM multi_escrows WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    if let Some(r) = row {
        let mut sigs: Vec<String> =
            serde_json::from_str(&r.try_get::<String, _>("signatures").unwrap_or_default())
                .unwrap_or_default();
        if !sigs.contains(&address.to_string()) {
            sigs.push(address.to_string());
        }
        sqlx::query("UPDATE multi_escrows SET signatures = ?1 WHERE id = ?2")
            .bind(serde_json::to_string(&sigs).unwrap_or_default())
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn settle_multi_escrow(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE multi_escrows SET status = 'settled', settled_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn refund_multi_escrow(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE multi_escrows SET status = 'refunded', refunded_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

fn row_to_multi(row: sqlx::sqlite::SqliteRow) -> MultiEscrow {
    MultiEscrow {
        id: row.try_get("id").unwrap_or_default(),
        lock_tx_id: row.try_get("lock_tx_id").unwrap_or_default(),
        parties: serde_json::from_str(
            &row.try_get::<String, _>("parties")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
        shares: serde_json::from_str(
            &row.try_get::<String, _>("shares")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
        total_amount: row.try_get("total_amount").unwrap_or(0),
        status: row.try_get("status").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        settled_at: row.try_get("settled_at").ok().flatten(),
        refunded_at: row.try_get("refunded_at").ok().flatten(),
        signatures: serde_json::from_str(
            &row.try_get::<String, _>("signatures")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
    }
}
