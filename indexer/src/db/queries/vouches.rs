use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_vouch(pool: &Pool<Sqlite>, vouch: &Vouch) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO vouches (id, voucher_address, subject_address, escrow_id, note, created_at, expires_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&vouch.id)
    .bind(&vouch.voucher_address)
    .bind(&vouch.subject_address)
    .bind(&vouch.escrow_id)
    .bind(&vouch.note)
    .bind(vouch.created_at)
    .bind(vouch.expires_at)
    .execute(pool).await?;
    Ok(())
}

pub async fn delete_vouch(
    pool: &Pool<Sqlite>,
    id: &str,
    voucher: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM vouches WHERE id = ?1 AND voucher_address = ?2")
        .bind(id)
        .bind(voucher)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_vouches_for_subject(
    pool: &Pool<Sqlite>,
    subject: &str,
) -> Result<Vec<Vouch>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let rows = sqlx::query(
        "SELECT * FROM vouches WHERE subject_address = ?1 AND expires_at > ?2 ORDER BY created_at DESC"
    )
    .bind(subject)
    .bind(now)
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(row_to_vouch).collect())
}

pub async fn list_vouches_by_voucher(
    pool: &Pool<Sqlite>,
    voucher: &str,
) -> Result<Vec<Vouch>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let rows = sqlx::query(
        "SELECT * FROM vouches WHERE voucher_address = ?1 AND expires_at > ?2 ORDER BY created_at DESC"
    )
    .bind(voucher)
    .bind(now)
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(row_to_vouch).collect())
}

pub async fn count_vouches_for_subject(
    pool: &Pool<Sqlite>,
    subject: &str,
) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vouches WHERE subject_address = ?1 AND expires_at > ?2",
    )
    .bind(subject)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn count_vouches_by_voucher(
    pool: &Pool<Sqlite>,
    voucher: &str,
) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM vouches WHERE voucher_address = ?1 AND expires_at > ?2",
    )
    .bind(voucher)
    .bind(now)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

/// Calculate weighted vouch score: weighted average of voucher reputations.
/// Vouchers with higher own reputation contribute more weight.
pub async fn calculate_vouch_score(
    pool: &Pool<Sqlite>,
    subject: &str,
) -> Result<Option<f64>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT voucher_address FROM vouches WHERE subject_address = ?1 AND expires_at > ?2",
    )
    .bind(subject)
    .bind(now)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    // Fetch scores for all voucher addresses in a single pass.
    // Use direct trade stats — NOT get_reputation — to avoid circular recursion.
    // Each voucher's score = f(trades, refunds, volume, age). Fetched via direct query.
    let mut total_weight = 0.0f64;
    let mut weighted_sum = 0.0f64;

    // Use a stored score cache: for each voucher, get their score directly
    // by querying their trade stats and calculating inline.
    // Since vouchers are addresses with 3+ trades (enforced at vouch creation),
    // this is always a small number of queries.
    for (voucher_addr,) in &rows {
        // Direct trade stats for this voucher (no vouch recursion)
        let (trade_c,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1",
        )
        .bind(voucher_addr)
        .fetch_one(pool)
        .await
        .unwrap_or((0,));
        let (refund_c,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded'"
        ).bind(voucher_addr).fetch_one(pool).await.unwrap_or((0,));
        let (vol,): (Option<i64>,) = sqlx::query_as(
            "SELECT SUM(amount_sompi) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'"
        ).bind(voucher_addr).fetch_one(pool).await.unwrap_or((None,));
        let (first,): (Option<i64>,) = sqlx::query_as(
            "SELECT MIN(created_at) FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1",
        )
        .bind(voucher_addr)
        .fetch_one(pool)
        .await
        .unwrap_or((None,));

        let age_days = first
            .map(|ts| ((chrono::Utc::now().timestamp() - ts).max(0) / 86_400).max(0))
            .unwrap_or(0);
        let volume = vol.unwrap_or(0);

        // Quick score (no recency — small approximation for vouch weighting)
        // Use simplified Beta with all-time data
        let total = trade_c.max(0) as f64;
        let score = if total < 1.0 {
            1.0
        } else {
            let failures = refund_c.max(0) as f64;
            let successes = (total - failures).max(0.0);
            let beta_raw = (successes + 1.0) / (successes + failures + 2.0);
            let centered = (beta_raw - 0.5) * 2.0;
            let volume_kas = (volume.max(0) as f64) / 100_000_000.0;
            let vol_bonus = (volume_kas / 1000.0 + 1.0).ln() * 0.12;
            let age_bonus = (age_days as f64 / 365.0).min(2.0) * 0.05;
            (1.0 + (centered * 4.0) + vol_bonus + age_bonus).clamp(1.0, 5.0)
        };

        let weight = score / 5.0;
        total_weight += weight;
        weighted_sum += weight * 4.0;
    }

    if total_weight < 0.01 {
        return Ok(None);
    }

    let vouch_score = (weighted_sum / total_weight).clamp(1.0, 5.0);
    Ok(Some(vouch_score))
}

pub(crate) fn row_to_vouch(row: sqlx::sqlite::SqliteRow) -> Vouch {
    Vouch {
        id: row.try_get("id").unwrap_or_default(),
        voucher_address: row.try_get("voucher_address").unwrap_or_default(),
        subject_address: row.try_get("subject_address").unwrap_or_default(),
        escrow_id: row.try_get("escrow_id").unwrap_or(None),
        note: row.try_get("note").unwrap_or(None),
        created_at: row.try_get("created_at").unwrap_or(0),
        expires_at: row.try_get("expires_at").unwrap_or(0),
    }
}
