use sqlx::{Pool, Sqlite, Row};
use serde::Serialize;

#[derive(Serialize)]
pub struct BlockedUser {
    pub id: String,
    pub blocker_address: String,
    pub blocked_address: String,
    pub reason: Option<String>,
    pub created_at: i64,
}

pub async fn create_block(
    pool: &Pool<Sqlite>,
    id: &str,
    blocker: &str,
    blocked: &str,
    reason: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO blocked_users (id, blocker_address, blocked_address, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5)"
    )
    .bind(id)
    .bind(blocker)
    .bind(blocked)
    .bind(reason)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_block(pool: &Pool<Sqlite>, id: &str, blocker: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blocked_users WHERE id = ?1 AND blocker_address = ?2")
        .bind(id)
        .bind(blocker)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_blocks(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Vec<BlockedUser>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, blocker_address, blocked_address, reason, created_at FROM blocked_users WHERE blocker_address = ?1 ORDER BY created_at DESC"
    )
    .bind(address)
    .fetch_all(pool)
    .await?;

    let blocks = rows.into_iter().map(|row| BlockedUser {
        id: row.get("id"),
        blocker_address: row.get("blocker_address"),
        blocked_address: row.get("blocked_address"),
        reason: row.get("reason"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(blocks)
}

pub async fn is_blocked(
    pool: &Pool<Sqlite>,
    blocker: &str,
    blocked: &str,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM blocked_users WHERE blocker_address = ?1 AND blocked_address = ?2"
    )
    .bind(blocker)
    .bind(blocked)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(count,)| count > 0).unwrap_or(false))
}
