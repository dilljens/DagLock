use chrono;
use sqlx::{Pool, Sqlite};

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

/// Helper: run a COUNT query across all escrow-like tables using UNION.
/// Each table must expose `buyer_address`, `seller_address`, `status`, `created_at` columns.
/// The `escrows` table has `buyer_address`/`seller_address` and `status` values like 'settled','refunded'.
/// New escrow types use their own tables with compatible column names.
async fn count_all_trades(
    pool: &Pool<Sqlite>,
    address: &str,
    ninety_days_ago: i64,
) -> Result<(i64, i64, i64, i64, i64, i64, Option<i64>), sqlx::Error> {
    // COUNT(*) across all escrow-like tables
    let (trade_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT buyer_address as a, seller_address as b FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1
            UNION ALL
            SELECT buyer_address as a, seller_address as b FROM milestone_escrows WHERE buyer_address = ?1 OR seller_address = ?1
            UNION ALL
            SELECT payer_address as a, recipient_address as b FROM subscriptions WHERE payer_address = ?1 OR recipient_address = ?1
            UNION ALL
            SELECT party1_address as a, party2_address as b FROM deposits WHERE party1_address = ?1 OR party2_address = ?1
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

    // Settled/completed count across all tables
    let (settled_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT status FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'
            UNION ALL
            SELECT status FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'completed'
            UNION ALL
            SELECT status FROM subscriptions WHERE (payer_address = ?1 OR recipient_address = ?1) AND status = 'completed'
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

    // Refunded/cancelled count
    let (refunded_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT status FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded'
            UNION ALL
            SELECT status FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded'
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

    // Recent trades (last 90 days)
    let (recent_trade_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT created_at FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND created_at >= ?2
            UNION ALL
            SELECT created_at FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND created_at >= ?2
            UNION ALL
            SELECT created_at FROM subscriptions WHERE (payer_address = ?1 OR recipient_address = ?1) AND created_at >= ?2
            UNION ALL
            SELECT created_at FROM deposits WHERE (party1_address = ?1 OR party2_address = ?1) AND created_at >= ?2
        )"
    ))
    .bind(address).bind(ninety_days_ago)
    .fetch_one(pool).await?;

    // Recent refunded
    let (recent_refunded_count,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT status, created_at FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded' AND created_at >= ?2
            UNION ALL
            SELECT status, created_at FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'refunded' AND created_at >= ?2
        )"
    ))
    .bind(address).bind(ninety_days_ago)
    .fetch_one(pool).await?;

    // Volume (SUM of settled/completed amounts)
    let (volume,): (Option<i64>,) = sqlx::query_as(&format!(
        "SELECT SUM(amount) FROM (
            SELECT amount_sompi as amount FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'
            UNION ALL
            SELECT total_amount as amount FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'completed'
            UNION ALL
            SELECT total_amount as amount FROM subscriptions WHERE (payer_address = ?1 OR recipient_address = ?1) AND status = 'completed'
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

    // First trade timestamp (oldest across all tables)
    let (first_trade_at,): (Option<i64>,) = sqlx::query_as(&format!(
        "SELECT MIN(created_at) FROM (
            SELECT created_at FROM escrows WHERE buyer_address = ?1 OR seller_address = ?1
            UNION ALL
            SELECT created_at FROM milestone_escrows WHERE buyer_address = ?1 OR seller_address = ?1
            UNION ALL
            SELECT created_at FROM subscriptions WHERE payer_address = ?1 OR recipient_address = ?1
            UNION ALL
            SELECT created_at FROM deposits WHERE party1_address = ?1 OR party2_address = ?1
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

    Ok((
        trade_count, settled_count, refunded_count,
        recent_trade_count, recent_refunded_count,
        volume.unwrap_or(0), first_trade_at,
    ))
}

pub async fn get_reputation(pool: &Pool<Sqlite>, address: &str) -> Result<Reputation, sqlx::Error> {
    let ninety_days_ago = chrono::Utc::now().timestamp() - 90 * 86_400;

    let (
        trade_count, settled_count, refunded_count,
        recent_trade_count, recent_refunded_count,
        total_volume, first_trade_at,
    ) = count_all_trades(pool, address, ninety_days_ago).await?;

    let age_days = first_trade_at
        .map(|ts| ((chrono::Utc::now().timestamp() - ts).max(0) / 86_400).max(0))
        .unwrap_or(0);

    // Dispute count: query escrows (disputed_at field) and milestone_escrows (status = 'disputed')
    let (disputed_count_raw,): (i64,) = sqlx::query_as(&format!(
        "SELECT COUNT(*) FROM (
            SELECT disputed_at FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND disputed_at IS NOT NULL
            UNION ALL
            SELECT created_at FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'disputed'
        )"
    ))
    .bind(address)
    .fetch_one(pool).await?;

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
    let telegram_handle = super::identity::get_identity_handle(pool, address, "telegram")
        .await
        .unwrap_or(None);

    // Fetch vouch stats
    let vouches_received = super::vouches::count_vouches_for_subject(pool, address)
        .await
        .unwrap_or(0);

    // Fetch mediator stats
    let mediator_stats = get_mediator_stats(pool, address).await.unwrap_or(None);

    // Wash trading signal: what fraction of volume is with a single counterparty?
    // Values > 0.9 mean this address almost exclusively trades with one other address,
    // which is a strong indicator of reputation farming.
    let trading_concentration = calculate_trading_concentration(pool, address)
        .await
        .unwrap_or(0.0);
    let vouches_given = super::vouches::count_vouches_by_voucher(pool, address)
        .await
        .unwrap_or(0);
    let vouch_score = super::vouches::calculate_vouch_score(pool, address)
        .await
        .unwrap_or(None);

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
        trading_concentration,
    })
}

pub async fn get_network_counts(pool: &Pool<Sqlite>) -> Result<(u64, u64, f64), sqlx::Error> {
    let (escrow_total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows")
        .fetch_one(pool).await?;
    let (milestone_total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM milestone_escrows")
        .fetch_one(pool).await?;
    let (sub_total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subscriptions")
        .fetch_one(pool).await?;
    let total = escrow_total.max(0) + milestone_total.max(0) + sub_total.max(0);

    let (escrow_settled,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM escrows WHERE status = 'settled'")
        .fetch_one(pool).await?;
    let (milestone_completed,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM milestone_escrows WHERE status = 'completed'")
        .fetch_one(pool).await?;
    let (sub_completed,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subscriptions WHERE status = 'completed'")
        .fetch_one(pool).await?;
    let settled = escrow_settled.max(0) + milestone_completed.max(0) + sub_completed.max(0);

    let avg_fee: (Option<f64>,) =
        sqlx::query_as("SELECT AVG(fee_sompi) FROM escrows WHERE fee_sompi > 0")
            .fetch_one(pool)
            .await?;

    Ok((
        total as u64,
        settled as u64,
        avg_fee.0.unwrap_or(0.0) / 100_000_000.0,
    ))
}

/// Calculate wash trading concentration: what fraction of volume
/// is with the single most-frequent counterparty?
/// Values > 0.9 indicate potential reputation farming.
pub async fn calculate_trading_concentration(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<f64, sqlx::Error> {
    // Get total volume with all counterparties across all escrow-like tables
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT counterparty, SUM(amount) as total FROM (
            SELECT CASE WHEN buyer_address = ?1 THEN seller_address ELSE buyer_address END as counterparty, amount_sompi as amount
             FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'settled'
            UNION ALL
            SELECT CASE WHEN buyer_address = ?1 THEN seller_address ELSE buyer_address END as counterparty, total_amount as amount
             FROM milestone_escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND status = 'completed'
            UNION ALL
            SELECT CASE WHEN payer_address = ?1 THEN recipient_address ELSE payer_address END as counterparty, total_amount as amount
             FROM subscriptions WHERE (payer_address = ?1 OR recipient_address = ?1) AND status = 'completed'
        ) GROUP BY counterparty"
    )
    .bind(address)
    .fetch_all(pool).await?;

    if rows.is_empty() {
        return Ok(0.0);
    }

    let total_volume: i64 = rows.iter().map(|(_, v)| v).sum();
    let max_volume = rows.iter().map(|(_, v)| v).max().unwrap_or(&0);

    if total_volume == 0 {
        return Ok(0.0);
    }

    Ok(*max_volume as f64 / total_volume as f64)
}

pub async fn get_mediator_stats(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Option<MediatorStats>, sqlx::Error> {
    // Count escrows where this address was the mediator
    let (cases_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE mediator_key = ?1 AND status != 'pending_confirmation'",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    if cases_count == 0 {
        return Ok(None);
    }

    // Count resolved (settled or refunded) — the ruling was accepted by both parties
    let (resolved_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE mediator_key = ?1 AND status IN ('settled', 'refunded')"
    ).bind(address).fetch_one(pool).await?;

    let (first_case_at,): (Option<i64>,) =
        sqlx::query_as("SELECT MIN(created_at) FROM escrows WHERE mediator_key = ?1")
            .bind(address)
            .fetch_one(pool)
            .await?;

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
