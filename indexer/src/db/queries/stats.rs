use sqlx::{Pool, Row, Sqlite};

use crate::types::{DailyStat, LiveSummary};

/// Aggregate today's data from all escrow-like tables and store a snapshot
/// into `daily_stats`. Uses INSERT OR REPLACE so calling it multiple times
/// per day overwrites the same row.
pub async fn compute_and_store_daily_stats(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let day_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|d| d.and_utc().timestamp())
        .unwrap_or(0);

    // Escrows created today
    let escrows_created: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE created_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Escrows settled today
    let escrows_settled: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE settled_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Milestones completed today
    let milestones_completed: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM milestone_escrows WHERE completed_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Subscriptions completed today
    let subscriptions_completed: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM subscriptions WHERE completed_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Volume (settled escrows today)
    let volume: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(amount_sompi) FROM escrows WHERE settled_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Fees (settled escrows today)
    let fees: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(fee_sompi) FROM escrows WHERE settled_at >= ?1",
    )
    .bind(day_start)
    .fetch_one(pool)
    .await?;

    // Active escrows (overall)
    let active_escrows: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE status IN ('active', 'pending_confirmation')",
    )
    .fetch_one(pool)
    .await?;

    // Open offers (overall)
    let open_offers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM offers WHERE status = 'proposed'",
    )
    .fetch_one(pool)
    .await?;

    // KAS/USD price from the in-memory cache
    let kas_usd_price = crate::types::fetch_kas_usd_price().await;

    // DAA score — just 0 when no wRPC connection
    let daa_score: i64 = 0;

    let created_at = now.timestamp();

    sqlx::query(
        "INSERT OR REPLACE INTO daily_stats
         (date, escrows_created, escrows_settled, milestones_completed,
          subscriptions_completed, volume_sompi, fees_sompi,
          active_escrows, open_offers, kas_usd_price, daa_score, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(&today)
    .bind(escrows_created.0)
    .bind(escrows_settled.0)
    .bind(milestones_completed.0)
    .bind(subscriptions_completed.0)
    .bind(volume.0.unwrap_or(0))
    .bind(fees.0.unwrap_or(0))
    .bind(active_escrows.0)
    .bind(open_offers.0)
    .bind(kas_usd_price)
    .bind(daa_score)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Return the last N days of daily stats, most recent first.
pub async fn get_daily_stats(
    pool: &Pool<Sqlite>,
    days: i64,
) -> Result<Vec<DailyStat>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT date, escrows_created, escrows_settled, volume_sompi, fees_sompi,
                active_escrows, open_offers, kas_usd_price
         FROM daily_stats
         ORDER BY date DESC
         LIMIT ?1",
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    let mut stats = Vec::with_capacity(rows.len());
    for row in rows {
        stats.push(DailyStat {
            date: row.try_get("date")?,
            escrows_created: row.try_get("escrows_created")?,
            escrows_settled: row.try_get("escrows_settled")?,
            volume_sompi: row.try_get("volume_sompi")?,
            fees_sompi: row.try_get("fees_sompi")?,
            active_escrows: row.try_get("active_escrows")?,
            open_offers: row.try_get("open_offers")?,
            kas_usd_price: row.try_get("kas_usd_price")?,
            total_users: 0,
        });
    }
    // Fill total_users for each day from escrows data
    if let Some(last) = stats.last_mut() {
        if let Ok((count,)) =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(DISTINCT buyer_address) FROM escrows")
                .fetch_one(pool)
                .await
        {
            last.total_users = count;
        }
    }

    Ok(stats)
}

/// Aggregate live totals across all tables.
pub async fn get_live_summary(pool: &Pool<Sqlite>) -> Result<LiveSummary, sqlx::Error> {
    let total_escrows: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows")
        .fetch_one(pool)
        .await?;

    let volume: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(amount_sompi) FROM escrows WHERE status = 'settled'",
    )
    .fetch_one(pool)
    .await?;

    let fees: (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(fee_sompi) FROM escrows WHERE status = 'settled'",
    )
    .fetch_one(pool)
    .await?;

    let active_escrows: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE status IN ('active', 'pending_confirmation')",
    )
    .fetch_one(pool)
    .await?;

    let total_users: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT buyer_address) FROM escrows",
    )
    .fetch_one(pool)
    .await?;

    let open_offers: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM offers WHERE status = 'proposed'",
    )
    .fetch_one(pool)
    .await?;

    Ok(LiveSummary {
        total_escrows: total_escrows.0,
        total_volume_sompi: volume.0.unwrap_or(0),
        total_fees_sompi: fees.0.unwrap_or(0),
        active_escrows: active_escrows.0,
        total_users: total_users.0,
        open_offers: open_offers.0,
        uptime_seconds: 0,
    })
}
