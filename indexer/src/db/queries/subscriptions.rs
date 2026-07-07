use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_subscription(
    pool: &Pool<Sqlite>,
    sub: &Subscription,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO subscriptions (id, payer_address, recipient_address, total_amount,
         installment_amount, interval_seconds, start_time, current_period, max_periods,
         status, created_at, cancelled_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )
    .bind(&sub.id)
    .bind(&sub.payer_address)
    .bind(&sub.recipient_address)
    .bind(sub.total_amount)
    .bind(sub.installment_amount)
    .bind(sub.interval_seconds)
    .bind(sub.start_time)
    .bind(sub.current_period)
    .bind(sub.max_periods)
    .bind(&sub.status)
    .bind(sub.created_at)
    .bind(sub.cancelled_at)
    .bind(sub.completed_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_subscription(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<Subscription>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM subscriptions WHERE id = ?1")
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(row_to_subscription).next())
}

pub async fn list_subscriptions_by_address(
    pool: &Pool<Sqlite>,
    address: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Subscription>, i64), sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM subscriptions
         WHERE payer_address = ?1 OR recipient_address = ?1
         ORDER BY created_at DESC
         LIMIT ?2 OFFSET ?3",
    )
    .bind(address)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM subscriptions
         WHERE payer_address = ?1 OR recipient_address = ?1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    let subscriptions: Vec<Subscription> = rows.into_iter().map(row_to_subscription).collect();
    Ok((subscriptions, count.0))
}

pub async fn mark_subscription_cancelled(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE subscriptions SET status = 'cancelled', cancelled_at = ?1 WHERE id = ?2 AND status = 'active'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_subscription_completed(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE subscriptions SET status = 'completed', completed_at = ?1 WHERE id = ?2 AND status = 'active'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Advance the current period counter after a successful draw.
pub async fn advance_subscription_period(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE subscriptions
         SET current_period = current_period + 1
         WHERE id = ?1 AND status = 'active' AND current_period < max_periods",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Find subscriptions due for an installment draw.
/// A subscription is due when:
/// - status = 'active'
/// - current_period < max_periods
/// - start_time + current_period * interval_seconds <= now
pub async fn find_due_subscriptions(
    pool: &Pool<Sqlite>,
    now_timestamp: i64,
) -> Result<Vec<Subscription>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM subscriptions
         WHERE status = 'active'
           AND current_period < max_periods
           AND (start_time + current_period * interval_seconds) <= ?1
         ORDER BY start_time ASC",
    )
    .bind(now_timestamp)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_subscription).collect())
}

fn row_to_subscription(row: sqlx::sqlite::SqliteRow) -> Subscription {
    Subscription {
        id: row.try_get("id").unwrap_or_default(),
        payer_address: row.try_get("payer_address").unwrap_or_default(),
        recipient_address: row.try_get("recipient_address").unwrap_or_default(),
        total_amount: row.try_get("total_amount").unwrap_or(0),
        installment_amount: row.try_get("installment_amount").unwrap_or(0),
        interval_seconds: row.try_get("interval_seconds").unwrap_or(0),
        start_time: row.try_get("start_time").unwrap_or(0),
        current_period: row.try_get("current_period").unwrap_or(0),
        max_periods: row.try_get("max_periods").unwrap_or(0),
        status: row.try_get("status").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        cancelled_at: row.try_get("cancelled_at").unwrap_or(None),
        completed_at: row.try_get("completed_at").unwrap_or(None),
    }
}
