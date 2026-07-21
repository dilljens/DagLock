use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_escrow(pool: &Pool<Sqlite>, escrow: &Escrow) -> Result<(), sqlx::Error> {
    sqlx::query(
          "INSERT INTO escrows (id, lock_tx_id, lock_tx_output_index, status, asset_type,
         buyer_address, seller_address, amount_sompi, fee_sompi, template_hash,
         expiration_daa_score, disputed_at, dispute_reason, cancelled_at, expired_at,
         created_at, settled_at, refunded_at, mediator_key, dispute_mode, dispute_outcome, dispute_resolved_at, price_at_creation, price_currency, trade_hash,
         price_lock_time, price_at_settlement, price_source, price_type, invoice_id, memo, auto_settle_timeout,
         mediation_status, mediation_buyer_claim, mediation_seller_claim, mediation_result, mediation_expires_at,
         mediation_buyer_accepted, mediation_seller_accepted,
         chat_pubkey_buyer, chat_pubkey_seller)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41)"
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
    .bind(&escrow.mediator_key).bind(&escrow.dispute_mode).bind(&escrow.dispute_outcome).bind(escrow.dispute_resolved_at).bind(escrow.price_at_creation).bind(&escrow.price_currency)
    .bind(&escrow.trade_hash)
    .bind(escrow.price_lock_time)
    .bind(escrow.price_at_settlement)
    .bind(&escrow.price_source)
    .bind(&escrow.price_type)
    .bind(&escrow.invoice_id)
    .bind(&escrow.memo)
    .bind(escrow.auto_settle_timeout)
    .bind(&escrow.mediation_status)
    .bind(&escrow.mediation_buyer_claim)
    .bind(&escrow.mediation_seller_claim)
    .bind(&escrow.mediation_result)
    .bind(escrow.mediation_expires_at)
    .bind(escrow.mediation_buyer_accepted)
    .bind(escrow.mediation_seller_accepted)
    .bind(&escrow.chat_pubkey_buyer)
    .bind(&escrow.chat_pubkey_seller)
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

    // Build WHERE clause — use unnumbered `?` for positional binding
    let where_clause = match role {
        "buyer" => "buyer_address = ?",
        "seller" => "seller_address = ?",
        _ => "(buyer_address = ? OR seller_address = ?)",
    };

    let status_clause = if status != "all" {
        " AND status = ?"
    } else {
        ""
    };

    let sql = format!(
        "SELECT * FROM escrows WHERE {where_clause}{status_clause} ORDER BY created_at DESC LIMIT ? OFFSET ?"
    );
    let count_sql = format!("SELECT COUNT(*) FROM escrows WHERE {where_clause}{status_clause}");

    // Execute data query — bind in order: address (1x or 2x for "all"), status, limit, offset
    let mut query = sqlx::query(&sql).bind(address);
    if role == "all" {
        // Bind address twice for the OR clause
        query = query.bind(address);
    }
    if status != "all" {
        query = query.bind(status);
    }
    let rows = query.bind(limit).bind(offset).fetch_all(pool).await?;

    // Execute count query
    let mut count_query = sqlx::query_as::<_, (i64,)>(&count_sql).bind(address);
    if role == "all" {
        count_query = count_query.bind(address);
    }
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
         WHERE id = ?2 AND status IN ('active', 'pending_confirmation')",
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

// ── wRPC Listener Queries

#[allow(dead_code)]
pub async fn try_find_escrow_by_lock_tx(
    pool: &Pool<Sqlite>,
    lock_tx_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let row =
        sqlx::query_as::<_, (String,)>("SELECT id FROM escrows WHERE lock_tx_id = ?1 LIMIT 1")
            .bind(lock_tx_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(id,)| id))
}

#[allow(dead_code)]
pub async fn update_escrow_status_only(
    pool: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE escrows SET status = ?1 WHERE id = ?2 AND status = 'pending_confirmation'")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub(crate) fn row_to_escrow(row: sqlx::sqlite::SqliteRow) -> Escrow {
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
    let dispute_mode: Option<String> = row.try_get("dispute_mode").unwrap_or(None);
    let dispute_outcome: Option<String> = row.try_get("dispute_outcome").unwrap_or(None);
    let dispute_resolved_at: Option<i64> = row.try_get("dispute_resolved_at").unwrap_or(None);
    let price_at_creation: Option<f64> = row.try_get("price_at_creation").unwrap_or(None);
    let price_currency: Option<String> = row.try_get("price_currency").unwrap_or(None);
    let trade_hash: Option<String> = row.try_get("trade_hash").ok().flatten();
    let price_lock_time: Option<i64> = row.try_get("price_lock_time").ok().flatten();
    let price_at_settlement: Option<f64> = row.try_get("price_at_settlement").ok().flatten();
    let price_source: Option<String> = row.try_get("price_source").ok().flatten();
    let price_type: Option<String> = row.try_get("price_type").ok().flatten();
    let invoice_id: Option<String> = row.try_get("invoice_id").ok().flatten();
    let memo: Option<String> = row.try_get("memo").ok().flatten();
    let auto_settle_timeout: Option<i64> = row.try_get("auto_settle_timeout").ok().flatten();
    let mediation_status: Option<String> = row.try_get("mediation_status").ok().flatten();
    let mediation_buyer_claim: Option<String> = row.try_get("mediation_buyer_claim").ok().flatten();
    let mediation_seller_claim: Option<String> =
        row.try_get("mediation_seller_claim").ok().flatten();
    let mediation_result: Option<String> = row.try_get("mediation_result").ok().flatten();
    let mediation_expires_at: Option<i64> = row.try_get("mediation_expires_at").ok().flatten();
    let mediation_buyer_accepted: Option<bool> =
        row.try_get("mediation_buyer_accepted").ok().flatten();
    let mediation_seller_accepted: Option<bool> =
        row.try_get("mediation_seller_accepted").ok().flatten();
    let chat_pubkey_buyer: Option<String> = row.try_get("chat_pubkey_buyer").ok().flatten();
    let chat_pubkey_seller: Option<String> = row.try_get("chat_pubkey_seller").ok().flatten();

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
        dispute_mode,
        dispute_outcome,
        dispute_resolved_at,
        price_at_creation,
        price_currency,
        trade_hash,
        price_lock_time,
        price_at_settlement,
        price_source,
        price_type,
        invoice_id,
        memo,
        auto_settle_timeout,
        mediation_status,
        mediation_buyer_claim,
        mediation_seller_claim,
        mediation_result,
        mediation_expires_at,
        mediation_buyer_accepted,
        mediation_seller_accepted,
        chat_pubkey_buyer,
        chat_pubkey_seller,
    }
}

/// Count escrows created by an address in the last N seconds.
/// Used for daily creation caps to prevent spam.
pub async fn count_escrows_by_buyer_recent(
    pool: &Pool<Sqlite>,
    buyer_address: &str,
    within_seconds: i64,
) -> Result<i64, sqlx::Error> {
    let cutoff = chrono::Utc::now().timestamp() - within_seconds;
    let row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM escrows WHERE buyer_address = ?1 AND created_at >= ?2",
    )
    .bind(buyer_address)
    .bind(cutoff)
    .fetch_one(pool)
    .await?;
    row.try_get("cnt")
}

/// Mark an escrow as auto-settled if it's active and the timeout has elapsed.
pub async fn auto_settle_escrow_atomic(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1
         WHERE id = ?2 AND status = 'active'
           AND auto_settle_timeout IS NOT NULL
           AND auto_settle_timeout <= ?1",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Find all escrows eligible for auto-settlement.
pub async fn find_auto_settleable_escrows(pool: &Pool<Sqlite>) -> Result<Vec<Escrow>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let rows = sqlx::query(
        "SELECT * FROM escrows WHERE status = 'active' AND auto_settle_timeout IS NOT NULL AND auto_settle_timeout <= ?1",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_escrow).collect())
}

/// Force settle a disputed escrow (used by mediation acceptance).
pub async fn force_settle_disputed(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1, dispute_resolved_at = ?1, dispute_outcome = 'mediation_refund'
         WHERE id = ?2 AND status = 'disputed'",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Force refund a disputed escrow (used by mediation acceptance).
pub async fn force_refund_disputed(pool: &Pool<Sqlite>, id: &str) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE escrows SET status = 'refunded', refunded_at = ?1, dispute_resolved_at = ?1, dispute_outcome = 'mediation_refund'
         WHERE id = ?2 AND status = 'disputed'",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Resolve a disputed escrow via split outcome.
pub async fn resolve_dispute_with_split(
    pool: &Pool<Sqlite>,
    id: &str,
    outcome: &str,
    _buyer_share_basis: i64,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE escrows SET status = 'settled', settled_at = ?1, dispute_resolved_at = ?1, dispute_outcome = ?2
         WHERE id = ?3 AND status = 'disputed'",
    )
    .bind(now)
    .bind(outcome)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Count recent state changes (settle/refund/dispute/cancel) for an address
/// within the last `window_secs` seconds. Used for per-address rate limiting
/// on state-changing operations.
pub async fn count_recent_state_changes(
    pool: &Pool<Sqlite>,
    address: &str,
    window_secs: i64,
) -> Result<i64, sqlx::Error> {
    let cutoff = chrono::Utc::now().timestamp() - window_secs;
    let row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM escrows WHERE \
         (buyer_address = ?1 OR seller_address = ?1) \
         AND (settled_at >= ?2 OR refunded_at >= ?2 OR disputed_at >= ?2 OR cancelled_at >= ?2)",
    )
    .bind(address)
    .bind(cutoff)
    .fetch_one(pool)
    .await?;
    let count: i64 = row.try_get("cnt").unwrap_or(0);
    Ok(count)
}
