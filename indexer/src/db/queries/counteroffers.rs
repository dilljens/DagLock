use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

#[derive(Debug, Serialize)]
pub struct CounterOffer {
    pub id: String,
    pub offer_id: String,
    pub proposer_address: String,
    pub amount_sompi: Option<i64>,
    pub price_offset: Option<f64>,
    pub timeout: Option<i64>,
    pub dispute_mode: Option<String>,
    pub message: Option<String>,
    pub status: String,
    pub created_at: i64,
}

pub async fn create_counteroffer(
    pool: &Pool<Sqlite>,
    id: &str,
    offer_id: &str,
    proposer: &str,
    amount_sompi: Option<i64>,
    price_offset: Option<f64>,
    timeout: Option<i64>,
    dispute_mode: Option<&str>,
    message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO offer_counteroffers \
         (id, offer_id, proposer_address, amount_sompi, price_offset, timeout, dispute_mode, message, status, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9)"
    )
    .bind(id)
    .bind(offer_id)
    .bind(proposer)
    .bind(amount_sompi)
    .bind(price_offset)
    .bind(timeout)
    .bind(dispute_mode)
    .bind(message)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_counteroffer(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<CounterOffer>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, offer_id, proposer_address, amount_sompi, price_offset, timeout, dispute_mode, message, status, created_at \
         FROM offer_counteroffers WHERE id = ?1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| CounterOffer {
        id: r.get("id"),
        offer_id: r.get("offer_id"),
        proposer_address: r.get("proposer_address"),
        amount_sompi: r.get("amount_sompi"),
        price_offset: r.get("price_offset"),
        timeout: r.get("timeout"),
        dispute_mode: r.get("dispute_mode"),
        message: r.get("message"),
        status: r.get("status"),
        created_at: r.get("created_at"),
    }))
}

pub async fn list_counteroffers(
    pool: &Pool<Sqlite>,
    offer_id: &str,
) -> Result<Vec<CounterOffer>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, offer_id, proposer_address, amount_sompi, price_offset, timeout, dispute_mode, message, status, created_at \
         FROM offer_counteroffers WHERE offer_id = ?1 ORDER BY created_at DESC"
    )
    .bind(offer_id)
    .fetch_all(pool)
    .await?;

    let offers = rows
        .into_iter()
        .map(|r| CounterOffer {
            id: r.get("id"),
            offer_id: r.get("offer_id"),
            proposer_address: r.get("proposer_address"),
            amount_sompi: r.get("amount_sompi"),
            price_offset: r.get("price_offset"),
            timeout: r.get("timeout"),
            dispute_mode: r.get("dispute_mode"),
            message: r.get("message"),
            status: r.get("status"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(offers)
}

pub async fn update_counteroffer_status(
    pool: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE offer_counteroffers SET status = ?1 WHERE id = ?2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Count pending counter-offers for an offer (for anti-spam: max 10).
pub async fn count_pending_for_offer(
    pool: &Pool<Sqlite>,
    offer_id: &str,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM offer_counteroffers WHERE offer_id = ?1 AND status = 'pending'",
    )
    .bind(offer_id)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
