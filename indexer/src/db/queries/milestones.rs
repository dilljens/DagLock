use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_milestone_escrow(
    pool: &Pool<Sqlite>,
    m: &MilestoneEscrow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO milestone_escrows (id, lock_tx_id, buyer_address, seller_address, total_amount, milestone_amounts, milestone_timeouts, current_milestone, milestone_statuses, status, created_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(&m.id)
    .bind(&m.lock_tx_id)
    .bind(&m.buyer_address)
    .bind(&m.seller_address)
    .bind(m.total_amount)
    .bind(serde_json::to_string(&m.milestone_amounts).unwrap_or_default())
    .bind(serde_json::to_string(&m.milestone_timeouts).unwrap_or_default())
    .bind(m.current_milestone)
    .bind(serde_json::to_string(&m.milestone_statuses).unwrap_or_default())
    .bind(&m.status)
    .bind(m.created_at)
    .bind(m.completed_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_milestone_escrow(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<MilestoneEscrow>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM milestone_escrows WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(row_to_milestone))
}

pub async fn list_milestones_by_address(
    pool: &Pool<Sqlite>,
    address: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<MilestoneEscrow>, i64), sqlx::Error> {
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milestone_escrows WHERE buyer_address = ?1 OR seller_address = ?1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        "SELECT * FROM milestone_escrows WHERE buyer_address = ?1 OR seller_address = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
    )
    .bind(address)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let milestones: Vec<MilestoneEscrow> = rows.into_iter().map(row_to_milestone).collect();
    Ok((milestones, total.0))
}

pub async fn update_milestone_status(
    pool: &Pool<Sqlite>,
    id: &str,
    current_milestone: i32,
    milestone_statuses: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE milestone_escrows SET current_milestone = ?1, milestone_statuses = ?2 WHERE id = ?3",
    )
    .bind(current_milestone)
    .bind(serde_json::to_string(milestone_statuses).unwrap_or_default())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_milestone_escrow(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "UPDATE milestone_escrows SET status = 'completed', completed_at = ?1 WHERE id = ?2",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn refund_milestone_escrow(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "UPDATE milestone_escrows SET status = 'refunded', completed_at = ?1 WHERE id = ?2",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

fn row_to_milestone(row: sqlx::sqlite::SqliteRow) -> MilestoneEscrow {
    MilestoneEscrow {
        id: row.try_get("id").unwrap_or_default(),
        lock_tx_id: row.try_get("lock_tx_id").unwrap_or_default(),
        buyer_address: row.try_get("buyer_address").unwrap_or_default(),
        seller_address: row.try_get("seller_address").unwrap_or_default(),
        total_amount: row.try_get("total_amount").unwrap_or(0),
        milestone_amounts: serde_json::from_str(
            &row.try_get::<String, _>("milestone_amounts")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
        milestone_timeouts: serde_json::from_str(
            &row.try_get::<String, _>("milestone_timeouts")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
        current_milestone: row.try_get("current_milestone").unwrap_or(0),
        milestone_statuses: serde_json::from_str(
            &row.try_get::<String, _>("milestone_statuses")
                .unwrap_or_else(|_| "[]".to_string()),
        )
        .unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        completed_at: row.try_get("completed_at").ok().flatten(),
    }
}
