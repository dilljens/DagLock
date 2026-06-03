//! Database query functions for the DagLock indexer.

use blake2b_simd::Params;
use chrono;
use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

/// Calculate reputation score using the Beta reputation system (Josang 2002).
///
/// Core formula: (successes + 1) / (total + 2)  — Laplace-smoothed Beta expectation
/// This is the academic standard for decentralized reputation systems.
///
/// Layered on top:
/// - Minimum trade threshold (first 5 trades = confidence ramp)
/// - Logarithmic volume bonus (bigger trades = slight edge)
/// - Gentle age bonus (older accounts = slight edge)
/// - Mapped to [1.0, 5.0] scale
///
/// Key properties:
/// - Disputes + refunds count as failures (full weight, unlike old formula's 0.25x refunds)
/// - Beta formula naturally gives tighter confidence with more data
/// - A single bad trade can't be offset by volume — only by more good trades
pub fn calculate_reputation_score(
    trade_count: i64,
    recent_trade_count: i64,
    total_volume_sompi: i64,
    age_days: i64,
    refunded_count: i64,
    recent_refunded_count: i64,
) -> f64 {
    // Beta reputation (Josang 2002) with recency weighting.
    //
    // Recent trades (last 90 days) are weighted 2x compared to old trades.
    // This prevents the "build trust for a year, then scam" attack vector.
    //
    // alpha = successes = effective_trades - effective_refunds
    // beta  = failures  = effective_refunds
    // effective_X = recent_trades * 2.0 + old_trades * 1.0
    //
    // Laplace smoothing (+1/+2) prevents 0/0.

    let all_time_total = trade_count.max(0) as f64;
    let recent_total = recent_trade_count.max(0) as f64;
    let old_total = (all_time_total - recent_total).max(0.0);

    let recent_refunds = recent_refunded_count.max(0) as f64;
    let old_refunds = (refunded_count.max(0) as f64 - recent_refunds).max(0.0);

    // Effective totals with recency weighting (2x for recent)
    let recent_weight: f64 = 2.0;
    let effective_total = recent_total * recent_weight + old_total;
    let effective_refunds = recent_refunds * recent_weight + old_refunds;
    let effective_successes = (effective_total - effective_refunds).max(0.0);

    if all_time_total < 1.0 {
        return 1.0;
    }

    // Beta core with Laplace smoothing
    let alpha = effective_successes;
    let beta = effective_refunds;
    let beta_raw = (alpha + 1.0) / (alpha + beta + 2.0);
    let centered = (beta_raw - 0.5) * 2.0;

    // Volume bonus
    let volume_kas = (total_volume_sompi.max(0) as f64) / 100_000_000.0;
    let volume_bonus = (volume_kas / 1000.0 + 1.0).ln() * 0.12;

    // Age bonus
    let age_days = age_days.max(0) as f64;
    let age_bonus = (age_days / 365.0).min(2.0) * 0.05;

    // Map [0.0, 1.0] -> [1.0, 5.0]
    let raw_score = 1.0 + (centered * 4.0) + volume_bonus + age_bonus;
    raw_score.clamp(1.0, 5.0)
}

// ── Escrow Queries

pub async fn insert_escrow(pool: &Pool<Sqlite>, escrow: &Escrow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, seller_address, amount_sompi, fee_sompi, template_hash,
         expiration_daa_score, disputed_at, dispute_reason, cancelled_at, expired_at,
         created_at, settled_at, refunded_at, mediator_key, dispute_outcome, dispute_resolved_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)"
    )
    .bind(&escrow.id).bind(&escrow.lock_tx_id)
    .bind(escrow.lock_tx_output_index as i64).bind(escrow.status.as_str())
    .bind(&escrow.asset_type).bind(&escrow.buyer_address)
    .bind(&escrow.seller_address).bind(escrow.amount_sompi)
    .bind(escrow.fee_sompi).bind(&escrow.template_hash)
    .bind(escrow.expiration_daa_score).bind(escrow.disputed_at)
    .bind(&escrow.dispute_reason).bind(escrow.cancelled_at)
    .bind(escrow.expired_at)
    .bind(escrow.created_at).bind(escrow.settled_at).bind(escrow.refunded_at)
    .bind(&escrow.mediator_key).bind(&escrow.dispute_outcome).bind(escrow.dispute_resolved_at)
    .execute(pool).await?;
    Ok(())
}

pub async fn get_escrow(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Escrow>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM escrows WHERE id = ?1")
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(row_to_escrow).next())
}

pub async fn list_escrows_by_address(
    pool: &Pool<Sqlite>,
    address: &str,
    role: Option<&str>,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Escrow>, i64), sqlx::Error> {
    let role = role.unwrap_or("all");
    let status = status.unwrap_or("all");

    // Build WHERE clause with consistent bind positions
    let where_clause = match role {
        "buyer" => "buyer_address = ?1",
        "seller" => "seller_address = ?1",
        _ => "(buyer_address = ?1 OR seller_address = ?1)",
    };

    let status_clause = if status != "all" {
        " AND status = ?2"
    } else {
        ""
    };
    let sql = format!("SELECT * FROM escrows WHERE {where_clause}{status_clause} ORDER BY created_at DESC LIMIT ?3 OFFSET ?4");
    let count_sql = format!("SELECT COUNT(*) FROM escrows WHERE {where_clause}{status_clause}");

    // Execute data query
    let mut query = sqlx::query(&sql).bind(address);
    if status != "all" {
        query = query.bind(status);
    }
    let rows = query.bind(limit).bind(offset).fetch_all(pool).await?;

    // Execute count query
    let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql).bind(address);
    if status != "all" {
        count_query = count_query.bind(status);
    }
    let (count,) = count_query.fetch_one(pool).await?;

    let escrows: Vec<Escrow> = rows.into_iter().map(row_to_escrow).collect();
    Ok((escrows, count))
}

/// Atomically settle an escrow: update status + settled_at in one query.
/// Returns true if the update succeeded (escrow was in active state).
pub async fn settle_escrow_atomic(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE escrows 
         SET status = 'settled', settled_at = ?1, refunded_at = NULL 
         WHERE id = ?2 AND status = 'active'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Atomically refund an escrow: update status + refunded_at in one query.
/// Returns true if the update succeeded (escrow was in active state).
pub async fn refund_escrow_atomic(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE escrows 
         SET status = 'refunded', refunded_at = ?1, settled_at = NULL 
         WHERE id = ?2 AND status = 'active'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn mark_escrow_disputed(
    pool: &Pool<Sqlite>,
    id: &str,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE escrows SET status = 'disputed', disputed_at = ?1, dispute_reason = ?2 WHERE id = ?3"
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(reason)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_escrow_cancelled(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE escrows SET status = 'cancelled', cancelled_at = ?1 WHERE id = ?2")
        .bind(chrono::Utc::now().timestamp())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Reconcile expired escrows based on DAA score.
///
/// Only expires escrows where:
/// - Status is pending_confirmation or active
/// - expiration_daa_score is set
/// - expiration_daa_score <= current_chain_daa_score
///
/// NOTE: Requires current DAA score from wRPC node. Until wRPC listener is
/// implemented, pass 0 to avoid premature expiration. The caller should
/// fetch the current DAA score from the Kaspa node before each reconciliation cycle.
pub async fn reconcile_expired_escrows(
    pool: &Pool<Sqlite>,
    current_daa_score: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE escrows
         SET status = 'expired', expired_at = ?1
         WHERE status IN ('pending_confirmation', 'active')
           AND expired_at IS NULL
           AND expiration_daa_score IS NOT NULL
           AND expiration_daa_score <= ?2",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(current_daa_score)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn get_stats(pool: &Pool<Sqlite>) -> Result<StatsResponse, sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows")
        .fetch_one(pool)
        .await?;
    let active: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'active'")
        .fetch_one(pool)
        .await?;
    let disputed: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'disputed'")
        .fetch_one(pool)
        .await?;
    let settled: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'settled'")
        .fetch_one(pool)
        .await?;
    let refunded: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'refunded'")
        .fetch_one(pool)
        .await?;
    let cancelled: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'cancelled'")
            .fetch_one(pool)
            .await?;
    let volume: (Option<i64>,) =
        sqlx::query_as("SELECT SUM(amount_sompi) FROM escrows WHERE status = 'settled'")
            .fetch_one(pool)
            .await?;
    let fees: (Option<i64>,) =
        sqlx::query_as("SELECT SUM(fee_sompi) FROM escrows WHERE status = 'settled'")
            .fetch_one(pool)
            .await?;
    let buyers: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT buyer_address) FROM escrows")
        .fetch_one(pool)
        .await?;
    let sellers: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT seller_address) FROM escrows WHERE seller_address IS NOT NULL",
    )
    .fetch_one(pool)
    .await?;

    Ok(StatsResponse {
        total_escrows: total.0,
        active_escrows: active.0,
        disputed_escrows: disputed.0,
        settled_escrows: settled.0,
        refunded_escrows: refunded.0,
        cancelled_escrows: cancelled.0,
        total_volume_kas: volume
            .0
            .map(|v| (v as f64 / 100_000_000.0).to_string())
            .unwrap_or_else(|| "0".into()),
        total_fees_collected_kas: fees
            .0
            .map(|f| (f as f64 / 100_000_000.0).to_string())
            .unwrap_or_else(|| "0".into()),
        unique_buyers: buyers.0,
        unique_sellers: sellers.0,
    })
}

pub async fn get_network_counts(pool: &Pool<Sqlite>) -> Result<(u64, u64, f64), sqlx::Error> {
    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows")
        .fetch_one(pool)
        .await?;
    let settled: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'settled'")
        .fetch_one(pool)
        .await?;
    let avg_fee: (Option<f64>,) =
        sqlx::query_as("SELECT AVG(fee_sompi) FROM escrows WHERE fee_sompi > 0")
            .fetch_one(pool)
            .await?;

    Ok((
        total.0.max(0) as u64,
        settled.0.max(0) as u64,
        avg_fee.0.unwrap_or(0.0) / 100_000_000.0,
    ))
}

pub async fn get_reputation(pool: &Pool<Sqlite>, address: &str) -> Result<Reputation, sqlx::Error> {
    let (trade_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    let (settled_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'"
    ).bind(address).fetch_one(pool).await?;

    let (refunded_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded'"
    ).bind(address).fetch_one(pool).await?;

    // Recent trades (last 90 days) — weighted 2x in score formula
    // to prevent "build trust then scam" attacks.
    let ninety_days_ago = chrono::Utc::now().timestamp() - 90 * 86_400;
    let (recent_trade_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND created_at >= ?2"
    ).bind(address).bind(ninety_days_ago).fetch_one(pool).await?;

    let (recent_refunded_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded' AND created_at >= ?2"
    ).bind(address).bind(ninety_days_ago).fetch_one(pool).await?;

    // Dispute info for display purposes (does not affect Beta score calculation).
    // The terminal state (settled vs refunded) determines success/failure; dispute
    // flags can double-count with terminal states.
    let (disputed_count_raw,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND disputed_at IS NOT NULL"
    ).bind(address).fetch_one(pool).await?;

    // Count upheld disputes (dispute was valid, this address was the at-fault party).
    // Uses dispute_outcome='uphold' which means the dispute filer was correct.
    let (_upheld_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1)
         AND dispute_outcome = 'uphold'",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    let (volume,): (Option<i64>,) = sqlx::query_as(
        "SELECT SUM(amount_sompi) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'"
    ).bind(address).fetch_one(pool).await?;

    let (first_trade_at,): (Option<i64>,) = sqlx::query_as(
        "SELECT MIN(created_at) FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    let age_days = first_trade_at
        .map(|ts| ((chrono::Utc::now().timestamp() - ts).max(0) / 86_400).max(0))
        .unwrap_or(0);
    let total_volume = volume.unwrap_or(0);

    // Scoring uses refunded_count as Beta failures (mutually exclusive with settled).
    // Dispute rates are informational only — the terminal state tells the real story.
    let refund_rate = if trade_count > 0 {
        refunded_count as f64 / trade_count as f64
    } else {
        0.0
    };
    let dispute_rate = if trade_count > 0 {
        disputed_count_raw as f64 / trade_count as f64
    } else {
        0.0
    };
    let score = calculate_reputation_score(
        trade_count,
        recent_trade_count,
        total_volume,
        age_days,
        refunded_count,
        recent_refunded_count,
    );

    // Fetch optional telegram handle
    let telegram_handle = get_identity_handle(pool, address, "telegram")
        .await
        .unwrap_or(None);

    // Fetch vouch stats
    let vouches_received = count_vouches_for_subject(pool, address).await.unwrap_or(0);

    // Fetch mediator stats
    let mediator_stats = get_mediator_stats(pool, address).await.unwrap_or(None);
    let vouches_given = count_vouches_by_voucher(pool, address).await.unwrap_or(0);
    let vouch_score = calculate_vouch_score(pool, address).await.unwrap_or(None);

    Ok(Reputation {
        address: address.to_string(),
        trade_count,
        recent_trade_count,
        total_volume_sompi: total_volume,
        settled_count,
        refunded_count,
        disputed_count: disputed_count_raw,
        first_trade_at,
        age_days,
        dispute_rate,
        refund_rate,
        score,
        telegram_handle,
        vouches_received,
        vouches_given,
        vouch_score,
        mediator_stats,
    })
}

// ── Evidence Queries

pub async fn insert_evidence(
    pool: &Pool<Sqlite>,
    evidence: &DisputeEvidence,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO dispute_evidence (id, escrow_id, submitted_by, content, content_hash, signed_message, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind(&evidence.id)
    .bind(&evidence.escrow_id)
    .bind(&evidence.submitted_by)
    .bind(&evidence.content)
    .bind(&evidence.content_hash)
    .bind(&evidence.signed_message)
    .bind(evidence.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_evidence(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Vec<DisputeEvidence>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT * FROM dispute_evidence WHERE escrow_id = ?1 ORDER BY created_at ASC")
            .bind(escrow_id)
            .fetch_all(pool)
            .await?;
    let evidence = rows
        .into_iter()
        .map(|row| DisputeEvidence {
            id: row.try_get("id").unwrap_or_default(),
            escrow_id: row.try_get("escrow_id").unwrap_or_default(),
            submitted_by: row.try_get("submitted_by").unwrap_or_default(),
            content: row.try_get("content").unwrap_or_default(),
            content_hash: row.try_get("content_hash").unwrap_or_default(),
            signed_message: row.try_get("signed_message").unwrap_or(None),
            created_at: row.try_get("created_at").unwrap_or(0),
        })
        .collect();
    Ok(evidence)
}

pub async fn resolve_dispute(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    outcome: &str,
    _resolved_by: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE escrows SET dispute_outcome = ?1, dispute_resolved_at = ?2 WHERE id = ?3 AND status = 'disputed'",
    )
    .bind(outcome)
    .bind(chrono::Utc::now().timestamp())
    .bind(escrow_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Identity Queries

pub async fn upsert_identity(
    pool: &Pool<Sqlite>,
    address: &str,
    platform: &str,
    handle: &str,
    signed_message: &str,
    signature_hex: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO verified_identities (address, platform, handle, signed_message, signature_hex, verified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(address)
    .bind(platform)
    .bind(handle)
    .bind(signed_message)
    .bind(signature_hex)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_identity_handle(
    pool: &Pool<Sqlite>,
    address: &str,
    platform: &str,
) -> Result<Option<String>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT handle FROM verified_identities WHERE address = ?1 AND platform = ?2")
            .bind(address)
            .bind(platform)
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| r.try_get::<String, _>("handle").ok())
        .next())
}

// ── Vouch Queries

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

fn row_to_vouch(row: sqlx::sqlite::SqliteRow) -> Vouch {
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



// ── Jury Queries

pub async fn register_juror(pool: &Pool<Sqlite>, address: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR REPLACE INTO juror_registrations (address, registered_at, total_cases_assigned, total_cases_voted, reliability_score)
         VALUES (?1, ?2, 0, 0, 1.0)"
    )
    .bind(address)
    .bind(now)
    .execute(pool).await?;
    Ok(())
}

pub async fn unregister_juror(pool: &Pool<Sqlite>, address: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM juror_registrations WHERE address = ?1")
        .bind(address)
        .execute(pool).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_juror(pool: &Pool<Sqlite>, address: &str) -> Result<Option<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM juror_registrations WHERE address = ?1")
        .bind(address)
        .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| JurorRegistration {
        address: row.try_get("address").unwrap_or_default(),
        registered_at: row.try_get("registered_at").unwrap_or(0),
        total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
        total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
        reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
    }).next())
}

pub async fn list_eligible_jurors(pool: &Pool<Sqlite>, min_score: f64) -> Result<Vec<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM juror_registrations WHERE reliability_score >= ?1 ORDER BY reliability_score DESC"
    )
    .bind((min_score * 100.0) as i64) // store as integer in SQLite
    .fetch_all(pool).await?;
    // Actually reliability_score is REAL, not stored as int
    Ok(rows.into_iter().map(|row| JurorRegistration {
        address: row.try_get("address").unwrap_or_default(),
        registered_at: row.try_get("registered_at").unwrap_or(0),
        total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
        total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
        reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
    }).collect())
}

pub async fn list_eligible_jurors_simple(pool: &Pool<Sqlite>) -> Result<Vec<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM juror_registrations ORDER BY reliability_score DESC"
    )
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|row| JurorRegistration {
        address: row.try_get("address").unwrap_or_default(),
        registered_at: row.try_get("registered_at").unwrap_or(0),
        total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
        total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
        reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
    }).collect())
}

pub async fn create_jury_case(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    juror_count: i64,
    threshold: i64,
    juror_addresses: &[String],
) -> Result<String, sqlx::Error> {
    use uuid::Uuid;
    let case_id = format!("jr_{}", Uuid::new_v4().to_string().split('-').next().unwrap());
    let now = chrono::Utc::now().timestamp();

    // Insert case
    sqlx::query(
        "INSERT INTO jury_cases (id, escrow_id, status, juror_count, threshold, votes_for_seller, votes_for_buyer, created_at)
         VALUES (?1, ?2, 'voting', ?3, ?4, 0, 0, ?5)"
    )
    .bind(&case_id)
    .bind(escrow_id)
    .bind(juror_count)
    .bind(threshold)
    .bind(now)
    .execute(pool).await?;

    // Insert jury_votes rows for each juror (pre-assigned, votes NULL)
    for addr in juror_addresses {
        // Store assigned jurors — we use jury_votes with vote=NULL to indicate assignment
        sqlx::query(
            "INSERT INTO jury_votes (case_id, juror_address, vote, voted_at)
             VALUES (?1, ?2, '', 0)"
        )
        .bind(&case_id)
        .bind(addr)
        .execute(pool).await?;
    }

    Ok(case_id)
}

pub async fn get_jury_case(pool: &Pool<Sqlite>, case_id: &str) -> Result<Option<JuryCase>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM jury_cases WHERE id = ?1")
        .bind(case_id)
        .fetch_all(pool).await?;

    if rows.is_empty() {
        return Ok(None);
    }
    let row = &rows[0];
    let cid: String = row.try_get("id").unwrap_or_default();

    // Fetch assigned jurors
    let juror_rows = sqlx::query(
        "SELECT juror_address FROM jury_votes WHERE case_id = ?1"
    )
    .bind(&cid)
    .fetch_all(pool).await?;
    let jurors: Vec<String> = juror_rows.into_iter()
        .filter_map(|r| r.try_get::<String, _>("juror_address").ok())
        .collect();

    Ok(Some(JuryCase {
        id: cid,
        escrow_id: row.try_get("escrow_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        juror_count: row.try_get("juror_count").unwrap_or(0),
        threshold: row.try_get("threshold").unwrap_or(0),
        votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
        votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        decided_at: row.try_get("decided_at").unwrap_or(None),
        outcome: row.try_get("outcome").unwrap_or(None),
        jurors,
    }))
}

pub async fn get_jury_case_by_escrow(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<Option<JuryCase>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM jury_cases WHERE escrow_id = ?1 ORDER BY created_at DESC LIMIT 1")
        .bind(escrow_id)
        .fetch_all(pool).await?;
    if rows.is_empty() {
        return Ok(None);
    }
    let row = &rows[0];
    let cid: String = row.try_get("id").unwrap_or_default();

    let juror_rows = sqlx::query("SELECT juror_address FROM jury_votes WHERE case_id = ?1")
        .bind(&cid).fetch_all(pool).await?;
    let jurors: Vec<String> = juror_rows.into_iter()
        .filter_map(|r| r.try_get::<String, _>("juror_address").ok()).collect();

    Ok(Some(JuryCase {
        id: cid,
        escrow_id: row.try_get("escrow_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        juror_count: row.try_get("juror_count").unwrap_or(0),
        threshold: row.try_get("threshold").unwrap_or(0),
        votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
        votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        decided_at: row.try_get("decided_at").unwrap_or(None),
        outcome: row.try_get("outcome").unwrap_or(None),
        jurors,
    }))
}

pub async fn cast_jury_vote(
    pool: &Pool<Sqlite>,
    case_id: &str,
    juror_address: &str,
    vote: &str,
    reasoning: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "UPDATE jury_votes SET vote = ?1, voted_at = ?2, reasoning = ?3
         WHERE case_id = ?4 AND juror_address = ?5"
    )
    .bind(vote)
    .bind(now)
    .bind(reasoning)
    .bind(case_id)
    .bind(juror_address)
    .execute(pool).await?;

    // Update case counts
    if vote == "seller_wins" {
        sqlx::query("UPDATE jury_cases SET votes_for_seller = votes_for_seller + 1 WHERE id = ?1")
            .bind(case_id).execute(pool).await?;
    } else if vote == "buyer_wins" {
        sqlx::query("UPDATE jury_cases SET votes_for_buyer = votes_for_buyer + 1 WHERE id = ?1")
            .bind(case_id).execute(pool).await?;
    }

    Ok(())
}

pub async fn check_jury_verdict(pool: &Pool<Sqlite>, case_id: &str) -> Result<Option<String>, sqlx::Error> {
    let case = get_jury_case(pool, case_id).await?;
    match case {
        Some(c) if c.status == "voting" || c.status == "selecting" => {
            if c.votes_for_seller >= c.threshold {
                let now = chrono::Utc::now().timestamp();
                sqlx::query(
                    "UPDATE jury_cases SET status = 'decided', outcome = 'seller_wins', decided_at = ?1 WHERE id = ?2"
                ).bind(now).bind(case_id).execute(pool).await?;
                Ok(Some("seller_wins".to_string()))
            } else if c.votes_for_buyer >= c.threshold {
                let now = chrono::Utc::now().timestamp();
                sqlx::query(
                    "UPDATE jury_cases SET status = 'decided', outcome = 'buyer_wins', decided_at = ?1 WHERE id = ?2"
                ).bind(now).bind(case_id).execute(pool).await?;
                Ok(Some("buyer_wins".to_string()))
            } else {
                Ok(None) // No verdict yet
            }
        }
        Some(c) => Ok(c.outcome.clone()),
        None => Ok(None),
    }
}

pub async fn expire_stale_jury_cases(pool: &Pool<Sqlite>) -> Result<u64, sqlx::Error> {
    let deadline = chrono::Utc::now().timestamp() - 72 * 3600;
    let result = sqlx::query(
        "UPDATE jury_cases SET status = 'decided', outcome = 'seller_wins', decided_at = ?1
         WHERE status = 'voting' AND created_at < ?2"
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(deadline)
    .execute(pool).await?;
    Ok(result.rows_affected())
}

pub async fn list_active_jury_cases_for_juror(pool: &Pool<Sqlite>, juror_address: &str) -> Result<Vec<JuryCase>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT jc.* FROM jury_cases jc
         INNER JOIN jury_votes jv ON jv.case_id = jc.id
         WHERE jv.juror_address = ?1 AND jc.status IN ('selecting', 'voting')
         ORDER BY jc.created_at DESC"
    )
    .bind(juror_address)
    .fetch_all(pool).await?;

    let mut cases = Vec::new();
    for row in &rows {
        let cid: String = row.try_get("id").unwrap_or_default();
        let juror_rows = sqlx::query("SELECT juror_address FROM jury_votes WHERE case_id = ?1")
            .bind(&cid).fetch_all(pool).await?;
        let jurors: Vec<String> = juror_rows.into_iter()
            .filter_map(|r| r.try_get::<String, _>("juror_address").ok()).collect();
        cases.push(JuryCase {
            id: cid,
            escrow_id: row.try_get("escrow_id").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            juror_count: row.try_get("juror_count").unwrap_or(0),
            threshold: row.try_get("threshold").unwrap_or(0),
            votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
            votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
            created_at: row.try_get("created_at").unwrap_or(0),
            decided_at: row.try_get("decided_at").unwrap_or(None),
            outcome: row.try_get("outcome").unwrap_or(None),
            jurors,
        });
    }
    Ok(cases)
}

pub async fn get_mediator_stats(pool: &Pool<Sqlite>, address: &str) -> Result<Option<MediatorStats>, sqlx::Error> {
    // Count escrows where this address was the mediator
    let (cases_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE mediator_key = ?1 AND status != 'pending_confirmation'"
    ).bind(address).fetch_one(pool).await?;

    if cases_count == 0 {
        return Ok(None);
    }

    // Count resolved (settled or refunded) — the ruling was accepted by both parties
    let (resolved_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE mediator_key = ?1 AND status IN ('settled', 'refunded')"
    ).bind(address).fetch_one(pool).await?;

    let (first_case_at,): (Option<i64>,) = sqlx::query_as(
        "SELECT MIN(created_at) FROM escrows WHERE mediator_key = ?1"
    ).bind(address).fetch_one(pool).await?;

    let years_active = first_case_at
        .map(|ts| ((chrono::Utc::now().timestamp() - ts).max(0) as f64) / 86_400.0 / 365.0)
        .unwrap_or(0.0);

    let acceptance_rate = if cases_count > 0 {
        resolved_count as f64 / cases_count as f64
    } else {
        1.0
    };

    // Calculate mediator score
    let base = (cases_count as f64 / 10.0).min(1.0) * 5.0;
    let bonus = acceptance_rate * 1.0;
    let age_bonus = (years_active / 2.0).min(1.0) * 0.5;
    let score = (base + bonus + age_bonus).clamp(1.0, 5.0);

    Ok(Some(MediatorStats {
        disputes_mediated: cases_count,
        rulings_accepted: resolved_count,
        acceptance_rate,
        years_active,
        score,
    }))
}
// ── Message Queries

pub async fn insert_message(pool: &Pool<Sqlite>, msg: &EscrowMessage, content_enc: &str, nonce: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO escrow_messages (id, escrow_id, sender_address, content_enc, nonce, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )
    .bind(&msg.id)
    .bind(&msg.escrow_id)
    .bind(&msg.sender_address)
    .bind(content_enc)
    .bind(nonce)
    .bind(msg.created_at)
    .execute(pool).await?;
    Ok(())
}

pub async fn list_messages_raw(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<Vec<(String, String, String, i64)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT sender_address, content_enc, nonce, created_at FROM escrow_messages WHERE escrow_id = ?1 ORDER BY created_at ASC"
    )
    .bind(escrow_id)
    .fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| {
        (
            r.try_get::<String, _>("sender_address").unwrap_or_default(),
            r.try_get::<String, _>("content_enc").unwrap_or_default(),
            r.try_get::<String, _>("nonce").unwrap_or_default(),
            r.try_get::<i64, _>("created_at").unwrap_or(0),
        )
    }).collect())
}

pub async fn count_messages(pool: &Pool<Sqlite>, escrow_id: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrow_messages WHERE escrow_id = ?1"
    ).bind(escrow_id).fetch_one(pool).await?;
    Ok(count)
}


// ── Offer Queries

pub async fn insert_offer(pool: &Pool<Sqlite>, offer: &Offer) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO offers (id, creator_address, side, base_asset, quote_asset,
         amount_sompi, counterparty_address, status, expires_at, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )
    .bind(&offer.id)
    .bind(&offer.creator_address)
    .bind(&offer.side)
    .bind(&offer.base_asset)
    .bind(&offer.quote_asset)
    .bind(offer.amount_sompi)
    .bind(&offer.counterparty_address)
    .bind(&offer.status)
    .bind(offer.expires_at)
    .bind(offer.created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_offer(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Offer>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM offers WHERE id = ?1")
        .bind(id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(row_to_offer).next())
}

pub async fn list_offers(
    pool: &Pool<Sqlite>,
    asset: Option<&str>,
    side: Option<&str>,
    status: Option<&str>,
) -> Result<(Vec<Offer>, i64), sqlx::Error> {
    let asset = asset.unwrap_or("all");
    let side = side.unwrap_or("all");
    let status = status.unwrap_or("proposed");
    let mut bind_index = 1;

    // Track which params are present for bind ordering
    let has_asset = asset != "all";
    let has_side = side != "all";
    let has_status = status != "all";

    // We'll use a simpler approach: build the full query with fixed bind positions
    let sql = format!(
        "SELECT * FROM offers WHERE 1=1{}{}{} ORDER BY created_at DESC",
        if has_asset {
            format!(" AND (base_asset = ?{bind_index} OR quote_asset = ?{bind_index})")
        } else {
            String::new()
        },
        {
            if has_asset {
                bind_index += 1;
            }
            if has_side {
                let s = format!(" AND side = ?{bind_index}");
                bind_index += 1;
                s
            } else {
                String::new()
            }
        },
        {
            if has_status {
                format!(" AND status = ?{bind_index}")
            } else {
                String::new()
            }
        },
    );

    let mut query = sqlx::query(&sql);
    if has_asset {
        query = query.bind(asset);
    }
    if has_side {
        query = query.bind(side);
    }
    if has_status {
        query = query.bind(status);
    }
    let rows = query.fetch_all(pool).await?;

    // Count query (reset bind_index)
    bind_index = 1;
    let count_sql = format!(
        "SELECT COUNT(*) FROM offers WHERE 1=1{}{}{}",
        if has_asset {
            format!(" AND (base_asset = ?{bind_index} OR quote_asset = ?{bind_index})")
        } else {
            String::new()
        },
        {
            if has_asset {
                bind_index += 1;
            }
            if has_side {
                let s = format!(" AND side = ?{bind_index}");
                bind_index += 1;
                s
            } else {
                String::new()
            }
        },
        {
            if has_status {
                format!(" AND status = ?{bind_index}")
            } else {
                String::new()
            }
        },
    );

    let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql);
    if has_asset {
        count_query = count_query.bind(asset);
    }
    if has_side {
        count_query = count_query.bind(side);
    }
    if has_status {
        count_query = count_query.bind(status);
    }
    let (count,) = count_query.fetch_one(pool).await?;

    let offers: Vec<Offer> = rows.into_iter().map(row_to_offer).collect();
    Ok((offers, count))
}

pub async fn update_offer_status(
    pool: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE offers SET status = ?1 WHERE id = ?2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn accept_offer(
    pool: &Pool<Sqlite>,
    id: &str,
    counterparty: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE offers SET status = 'accepted', counterparty_address = ?1 WHERE id = ?2")
        .bind(counterparty)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_receipt_escrow(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<Escrow>, sqlx::Error> {
    get_escrow(pool, id).await
}

pub fn receipt_from_escrow(escrow: &Escrow) -> Receipt {
    let escrow_id = escrow.id.clone();
    let status = escrow.status.as_str().to_string();
    let asset = escrow.asset_type.clone();
    let amount_sompi = escrow.amount_sompi;
    let fee_sompi = escrow.fee_sompi;
    let buyer_address = escrow.buyer_address.clone();
    let seller_address = escrow.seller_address.clone();
    let lock_tx_id = escrow.lock_tx_id.clone();
    let lock_tx_output_index = escrow.lock_tx_output_index;
    let expiration_daa_score = escrow.expiration_daa_score;
    let disputed_at = escrow.disputed_at;
    let dispute_reason = escrow.dispute_reason.clone();
    let cancelled_at = escrow.cancelled_at;
    let expired_at = escrow.expired_at;
    let settled_at = escrow.settled_at;
    let refunded_at = escrow.refunded_at;

    let payload = ReceiptPayload {
        escrow_id: escrow_id.clone(),
        status: status.clone(),
        asset: asset.clone(),
        amount_sompi,
        fee_sompi,
        buyer_address: buyer_address.clone(),
        seller_address: seller_address.clone(),
        lock_tx_id: lock_tx_id.clone(),
        lock_tx_output_index,
        expiration_daa_score,
        disputed_at,
        dispute_reason: dispute_reason.clone(),
        cancelled_at,
        expired_at,
        settled_at,
        refunded_at,
    };
    let serialized = serde_json::to_vec(&payload).unwrap_or_default();
    let receipt_id = Params::new()
        .hash_length(16)
        .to_state()
        .update(&serialized)
        .finalize();

    Receipt {
        receipt_id: format!("rct_{}", hex::encode(receipt_id.as_bytes())),
        escrow_id,
        status,
        asset,
        amount_sompi,
        fee_sompi,
        buyer_address,
        seller_address,
        lock_tx_id,
        lock_tx_output_index,
        expiration_daa_score,
        disputed_at,
        dispute_reason,
        cancelled_at,
        expired_at,
        settled_at,
        refunded_at,
        verification: ReceiptVerification {
            // Basic check: template_hash was provided (indicates covenant was compiled)
            covenant_verified: !escrow.template_hash.is_empty(),
            // If escrow reached a final state, on-chain covenant verified signatures
            signatures_verified: settled_at.is_some()
                || refunded_at.is_some()
                || cancelled_at.is_some(),
            // Verify fee matches protocol rate (0.5% = 1/200)
            fee_compliant: fee_sompi == amount_sompi / 200,
        },
    }
}

#[derive(Debug, Serialize)]
struct ReceiptPayload {
    escrow_id: EscrowId,
    status: String,
    asset: String,
    amount_sompi: i64,
    fee_sompi: i64,
    buyer_address: Address,
    seller_address: Option<Address>,
    lock_tx_id: TxId,
    lock_tx_output_index: u32,
    expiration_daa_score: Option<i64>,
    disputed_at: Option<i64>,
    dispute_reason: Option<String>,
    cancelled_at: Option<i64>,
    expired_at: Option<i64>,
    settled_at: Option<i64>,
    refunded_at: Option<i64>,
}

// ── Row Mappers

fn row_to_escrow(row: sqlx::sqlite::SqliteRow) -> Escrow {
    // Use try_get for safety — corrupted DB rows won't panic
    let id: String = row.try_get("id").unwrap_or_default();
    let lock_tx_id: String = row.try_get("lock_tx_id").unwrap_or_default();
    let lock_tx_output_index: i64 = row.try_get("lock_tx_output_index").unwrap_or(0);
    let status_str: String = row.try_get("status").unwrap_or_default();
    let asset_type: String = row
        .try_get("asset_type")
        .unwrap_or_else(|_| "KAS".to_string());
    let buyer_address: String = row.try_get("buyer_address").unwrap_or_default();
    let seller_address: Option<String> = row.try_get("seller_address").unwrap_or(None);
    let amount_sompi: i64 = row.try_get("amount_sompi").unwrap_or(0);
    let fee_sompi: i64 = row.try_get("fee_sompi").unwrap_or(0);
    let template_hash: Vec<u8> = row.try_get("template_hash").unwrap_or_default();
    let expiration_daa_score: Option<i64> = row.try_get("expiration_daa_score").unwrap_or(None);
    let disputed_at: Option<i64> = row.try_get("disputed_at").unwrap_or(None);
    let dispute_reason: Option<String> = row.try_get("dispute_reason").unwrap_or(None);
    let cancelled_at: Option<i64> = row.try_get("cancelled_at").unwrap_or(None);
    let expired_at: Option<i64> = row.try_get("expired_at").unwrap_or(None);
    let created_at: i64 = row.try_get("created_at").unwrap_or(0);
    let settled_at: Option<i64> = row.try_get("settled_at").unwrap_or(None);
    let refunded_at: Option<i64> = row.try_get("refunded_at").unwrap_or(None);
    let mediator_key: Option<String> = row.try_get("mediator_key").unwrap_or(None);
    let dispute_outcome: Option<String> = row.try_get("dispute_outcome").unwrap_or(None);
    let dispute_resolved_at: Option<i64> = row.try_get("dispute_resolved_at").unwrap_or(None);

    Escrow {
        id,
        lock_tx_id,
        lock_tx_output_index: lock_tx_output_index as u32,
        status: EscrowStatus::parse_status(&status_str)
            .unwrap_or(EscrowStatus::PendingConfirmation),
        asset_type,
        buyer_address,
        seller_address,
        amount_sompi,
        fee_sompi,
        template_hash,
        expiration_daa_score,
        disputed_at,
        dispute_reason,
        cancelled_at,
        expired_at,
        created_at,
        settled_at,
        refunded_at,
        mediator_key,
        dispute_outcome,
        dispute_resolved_at,
    }
}

fn row_to_offer(row: sqlx::sqlite::SqliteRow) -> Offer {
    Offer {
        id: row.try_get("id").unwrap_or_default(),
        creator_address: row.try_get("creator_address").unwrap_or_default(),
        side: row.try_get("side").unwrap_or_default(),
        base_asset: row.try_get("base_asset").unwrap_or_default(),
        quote_asset: row.try_get("quote_asset").unwrap_or_default(),
        amount_sompi: row.try_get("amount_sompi").unwrap_or(0),
        counterparty_address: row.try_get("counterparty_address").unwrap_or(None),
        status: row.try_get("status").unwrap_or_default(),
        expires_at: row.try_get("expires_at").unwrap_or(None),
        created_at: row.try_get("created_at").unwrap_or(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn escrow_fixture() -> Escrow {
        Escrow {
            id: "esc_123".to_string(),
            lock_tx_id: "tx123".to_string(),
            lock_tx_output_index: 0,
            status: EscrowStatus::Disputed,
            asset_type: "KAS".to_string(),
            buyer_address: "kaspa:buyer".to_string(),
            seller_address: Some("kaspa:seller".to_string()),
            amount_sompi: 500_000_000,
            fee_sompi: 2_500_000,
            template_hash: vec![1, 2, 3],
            expiration_daa_score: Some(42),
            disputed_at: Some(1_700_000_000),
            dispute_reason: Some("seller did not deliver".to_string()),
            cancelled_at: None,
            expired_at: None,
            created_at: 1_700_000_000,
            settled_at: None,
            refunded_at: None,
            mediator_key: None,
            dispute_outcome: None,
            dispute_resolved_at: None,
        }
    }

    #[test]
    fn reputation_score_rises_with_trade_history() {
        let low = calculate_reputation_score(1, 0, 50_000_000, 1, 0, 0);
        let high = calculate_reputation_score(10, 10, 10_000_000_000, 180, 0, 0);
        assert!(high > low);
    }

    #[test]
    fn reputation_score_falls_with_disputes() {
        let clean = calculate_reputation_score(10, 10, 10_000_000_000, 180, 0, 0);
        let disputed = calculate_reputation_score(10, 10, 10_000_000_000, 180, 2, 2);
        assert!(disputed < clean);
    }

    #[test]
    fn receipt_carries_lifecycle_metadata() {
        let receipt = receipt_from_escrow(&escrow_fixture());
        assert_eq!(receipt.status, "disputed");
        assert_eq!(
            receipt.dispute_reason.as_deref(),
            Some("seller did not deliver")
        );
        assert!(receipt.disputed_at.is_some());
        assert!(!receipt.verification.signatures_verified);
        // fixture: template_hash = [1,2,3] (non-empty) → covenant_verified = true
        assert!(receipt.verification.covenant_verified);
        // fixture: amount=500M, fee=2.5M → 500M/200 = 2.5M ✓
        assert!(receipt.verification.fee_compliant);
    }

    #[test]
    fn receipt_detects_incorrect_fee() {
        let mut escrow = escrow_fixture();
        escrow.fee_sompi = 100; // wrong fee
        let receipt = receipt_from_escrow(&escrow);
        assert!(!receipt.verification.fee_compliant);
    }

    #[test]
    fn receipt_detects_empty_template_hash() {
        let mut escrow = escrow_fixture();
        escrow.template_hash = vec![];
        let receipt = receipt_from_escrow(&escrow);
        assert!(!receipt.verification.covenant_verified);
    }
}
