use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

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

pub async fn delete_block(
    pool: &Pool<Sqlite>,
    id: &str,
    blocker: &str,
) -> Result<bool, sqlx::Error> {
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

    let blocks = rows
        .into_iter()
        .map(|row| BlockedUser {
            id: row.get("id"),
            blocker_address: row.get("blocker_address"),
            blocked_address: row.get("blocked_address"),
            reason: row.get("reason"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(blocks)
}

pub async fn list_all_blocks(
    pool: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<Vec<BlockedUser>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, i64)>(
        "SELECT id, blocker_address, blocked_address, reason, created_at FROM blocked_users ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let blocks = rows
        .into_iter()
        .map(|r| BlockedUser {
            id: r.0,
            blocker_address: r.1,
            blocked_address: r.2,
            reason: r.3,
            created_at: r.4,
        })
        .collect();

    Ok(blocks)
}

pub async fn delete_block_by_id(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM blocked_users WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn is_blocked(
    pool: &Pool<Sqlite>,
    blocker: &str,
    blocked: &str,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM blocked_users WHERE blocker_address = ?1 AND blocked_address = ?2",
    )
    .bind(blocker)
    .bind(blocked)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(count,)| count > 0).unwrap_or(false))
}
