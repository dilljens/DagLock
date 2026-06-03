//! Database query functions for the DagLock indexer.

use blake2b_simd::Params;
use chrono;
use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub fn calculate_reputation_score(
    trade_count: i64,
    total_volume_sompi: i64,
    age_days: i64,
    disputed_count: i64,
    refunded_count: i64,
) -> f64 {
    if trade_count <= 0 {
        return 1.0;
    }

    let trade_component = ((trade_count as f64) + 1.0).ln();
    let volume_component = ((total_volume_sompi.max(0) as f64 / 100_000_000.0) + 1.0).ln();
    let dispute_rate = (disputed_count.max(0) as f64 / trade_count as f64).clamp(0.0, 1.0);
    let refund_rate = (refunded_count.max(0) as f64 / trade_count as f64).clamp(0.0, 1.0);
    let age_factor = ((age_days.max(0) as f64) / 30.0).clamp(0.25, 1.75);
    let quality_factor = (1.0 - dispute_rate).powf(2.0) * (1.0 - (refund_rate * 0.25));

    let raw = (trade_component + volume_component) * age_factor * quality_factor;
    (1.0 + (raw / 3.0)).clamp(1.0, 5.0)
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

    // Count upheld disputes where this address was the defendant (the one found at fault)
    // An upheld dispute means the dispute was valid — the defendant harmed the other party.
    // We count escrows where the outcome is 'upheld' and this address is one of the parties.
    let (upheld_against,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1)
         AND dispute_outcome = 'expunge'",
    )
    .bind(address)
    .fetch_one(pool)
    .await?;

    // Count expunged disputes (false disputes) where this address filed
    // An expunged dispute means the filer was wrong — they get a penalty.
    // We approximate by counting disputes with outcome='uphold' where this
    // address was any party (they were the wronged party, beneficiary).
    let (disputed_count_raw,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM escrows WHERE (buyer_address = ?1 OR seller_address = ?1) AND disputed_at IS NOT NULL"
    ).bind(address).fetch_one(pool).await?;

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

    // Adjust dispute count: upheld-against (expunged) disputes are subtracted from
    // the raw count because they were false disputes filed against this address.
    let effective_disputed_count = (disputed_count_raw - upheld_against).max(0);
    let refund_rate = if trade_count > 0 {
        refunded_count as f64 / trade_count as f64
    } else {
        0.0
    };
    let dispute_rate = if trade_count > 0 {
        effective_disputed_count as f64 / trade_count as f64
    } else {
        0.0
    };
    let score = calculate_reputation_score(
        trade_count,
        total_volume,
        age_days,
        effective_disputed_count,
        refunded_count,
    );

    // Fetch optional telegram handle
    let telegram_handle = get_identity_handle(pool, address, "telegram")
        .await
        .unwrap_or(None);

    Ok(Reputation {
        address: address.to_string(),
        trade_count,
        total_volume_sompi: total_volume,
        settled_count,
        refunded_count,
        disputed_count: effective_disputed_count,
        first_trade_at,
        age_days,
        dispute_rate,
        refund_rate,
        score,
        telegram_handle,
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
    let rows = sqlx::query(
        "SELECT handle FROM verified_identities WHERE address = ?1 AND platform = ?2",
    )
    .bind(address)
    .bind(platform)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.try_get::<String, _>("handle").ok()).next())
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
        let low = calculate_reputation_score(1, 50_000_000, 1, 0, 0);
        let high = calculate_reputation_score(10, 10_000_000_000, 180, 0, 0);
        assert!(high > low);
    }

    #[test]
    fn reputation_score_falls_with_disputes() {
        let clean = calculate_reputation_score(10, 10_000_000_000, 180, 0, 0);
        let disputed = calculate_reputation_score(10, 10_000_000_000, 180, 3, 2);
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
