use blake2b_simd::Params;
use chrono;
use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_offer(pool: &Pool<Sqlite>, offer: &Offer) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO offers (id, creator_address, side, base_asset, quote_asset,
         amount_sompi, counterparty_address, status, expires_at, created_at,
         price_type, price_offset, min_price, max_price, current_price, price_currency, price_updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
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
    .bind(&offer.price_type)
    .bind(offer.price_offset)
    .bind(offer.min_price)
    .bind(offer.max_price)
    .bind(offer.current_price)
    .bind(&offer.price_currency)
    .bind(offer.price_updated_at)
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

pub async fn list_offers_by_creator(
    pool: &Pool<Sqlite>,
    creator: &str,
) -> Result<(Vec<Offer>, i64), sqlx::Error> {
    let rows =
        sqlx::query("SELECT * FROM offers WHERE creator_address = ?1 ORDER BY created_at DESC")
            .bind(creator)
            .fetch_all(pool)
            .await?;
    let count = rows.len() as i64;
    let offers: Vec<Offer> = rows.into_iter().map(row_to_offer).collect();
    Ok((offers, count))
}

/// Reconcile expired offers: mark as expired if expires_at < now
#[allow(dead_code)]
pub async fn reconcile_expired_offers(pool: &Pool<Sqlite>) -> Result<u64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE offers SET status = 'expired' WHERE status = 'proposed' AND expires_at IS NOT NULL AND expires_at < ?1"
    )
    .bind(now)
    .execute(pool).await?;
    Ok(result.rows_affected())
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
    super::escrows::get_escrow(pool, id).await
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
        price_type: row.try_get("price_type").unwrap_or_default(),
        price_offset: row.try_get("price_offset").ok().flatten(),
        min_price: row.try_get("min_price").ok().flatten(),
        max_price: row.try_get("max_price").ok().flatten(),
        current_price: row.try_get("current_price").ok().flatten(),
        price_currency: row
            .try_get("price_currency")
            .unwrap_or_else(|_| "USD".to_string()),
        price_updated_at: row.try_get("price_updated_at").ok().flatten(),
    }
}

/// Count offers created by an address in the last N seconds.
/// Used for daily creation caps to prevent spam.
pub async fn count_offers_by_creator_recent(
    pool: &Pool<Sqlite>,
    creator_address: &str,
    within_seconds: i64,
) -> Result<i64, sqlx::Error> {
    let cutoff = chrono::Utc::now().timestamp() - within_seconds;
    let row = sqlx::query(
        "SELECT COUNT(*) as cnt FROM offers WHERE creator_address = ?1 AND created_at >= ?2 AND status = 'proposed'",
    )
    .bind(creator_address)
    .bind(cutoff)
    .fetch_one(pool)
    .await?;
    row.try_get("cnt")
}
