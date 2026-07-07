use sqlx::{Pool, Sqlite};
use std::time::Duration;
use tracing::{error, info, warn};

/// Spawn a background task that fetches KAS/USD from CoinGecko every 5 minutes
/// and stores it in the `price_history` table. Prunes data older than 90 days.
pub fn spawn(pool: Pool<Sqlite>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        // Run once on startup
        record_price(&pool).await;
        loop {
            interval.tick().await;
            record_price(&pool).await;
        }
    });
}

async fn record_price(pool: &Pool<Sqlite>) {
    let price = match crate::types::fetch_kas_usd_price().await {
        Some(p) => p,
        None => {
            warn!("Price oracle: CoinGecko fetch returned None, skipping");
            return;
        }
    };

    let now = chrono::Utc::now().timestamp();
    if let Err(e) = sqlx::query(
        "INSERT OR REPLACE INTO price_history (timestamp, price_usd, source) VALUES (?1, ?2, 'coingecko')"
    )
    .bind(now)
    .bind(price)
    .execute(pool)
    .await
    {
        error!("Price oracle: failed to store price: {e}");
        return;
    }

    info!("Price oracle: recorded KAS/USD = ${price:.4}");

    // Prune data older than 90 days
    let cutoff = now - 90 * 86400;
    if let Err(e) = sqlx::query("DELETE FROM price_history WHERE timestamp < ?1")
        .bind(cutoff)
        .execute(pool)
        .await
    {
        warn!("Price oracle: failed to prune history: {e}");
    }
}
