use sqlx::{Pool, Sqlite, Row};
use serde::Serialize;

#[derive(Serialize)]
pub struct TradeFeedbackRow {
    pub id: String,
    pub escrow_id: String,
    pub reviewer_address: String,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: i64,
}

pub struct FeedbackStats {
    pub average_rating: f64,
    pub total: i64,
}

pub async fn upsert_feedback(
    pool: &Pool<Sqlite>,
    id: &str,
    escrow_id: &str,
    reviewer: &str,
    rating: i32,
    comment: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO trade_feedback (id, escrow_id, reviewer_address, rating, comment, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT(escrow_id, reviewer_address) DO UPDATE SET rating = ?4, comment = ?5, created_at = ?6"
    )
    .bind(id)
    .bind(escrow_id)
    .bind(reviewer)
    .bind(rating)
    .bind(comment)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_feedback_for_escrow(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<TradeFeedbackRow>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, escrow_id, reviewer_address, rating, comment, created_at \
         FROM trade_feedback WHERE escrow_id = ?1 ORDER BY created_at DESC"
    )
    .bind(escrow_id)
    .fetch_all(pool)
    .await?;

    let feedback = rows.into_iter().map(|row| TradeFeedbackRow {
        id: row.get("id"),
        escrow_id: row.get("escrow_id"),
        reviewer_address: row.get("reviewer_address"),
        rating: row.get("rating"),
        comment: row.get("comment"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(feedback)
}

pub async fn get_feedback_stats(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<FeedbackStats, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COALESCE(AVG(CAST(rating AS REAL)), 0) as average_rating, \
         COUNT(*) as total \
         FROM trade_feedback tf \
         JOIN escrows e ON tf.escrow_id = e.id \
         WHERE (e.buyer_address = ?1 OR e.seller_address = ?1)"
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    Ok(FeedbackStats {
        average_rating: row.get("average_rating"),
        total: row.get("total"),
    })
}
