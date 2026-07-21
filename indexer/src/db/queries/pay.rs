use sqlx::{Pool, Row, Sqlite};

use crate::types::PaymentSession;

pub async fn insert_session(
    pool: &Pool<Sqlite>,
    session: &PaymentSession,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO payment_sessions (id, app_id, escrow_id, amount_sompi, asset_type, seller_address, memo, status, buyer_address, created_at, expires_at, webhook_url, redirect_url)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )
    .bind(&session.id)
    .bind(&session.app_id)
    .bind(&session.escrow_id)
    .bind(session.amount_sompi)
    .bind(&session.asset_type)
    .bind(&session.seller_address)
    .bind(&session.memo)
    .bind(&session.status)
    .bind(&session.buyer_address)
    .bind(session.created_at)
    .bind(session.expires_at)
    .bind(&session.webhook_url)
    .bind(&session.redirect_url)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_session(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<Option<PaymentSession>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, app_id, escrow_id, amount_sompi, asset_type, seller_address, memo, status, buyer_address, created_at, expires_at, webhook_url, redirect_url
         FROM payment_sessions WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => Ok(Some(PaymentSession {
            id: r.try_get("id")?,
            app_id: r.try_get("app_id")?,
            escrow_id: r.try_get("escrow_id")?,
            amount_sompi: r.try_get("amount_sompi")?,
            asset_type: r.try_get("asset_type")?,
            seller_address: r.try_get("seller_address")?,
            memo: r.try_get("memo")?,
            status: r.try_get("status")?,
            buyer_address: r.try_get("buyer_address")?,
            created_at: r.try_get("created_at")?,
            expires_at: r.try_get("expires_at")?,
            webhook_url: r.try_get("webhook_url")?,
            redirect_url: r.try_get("redirect_url")?,
        })),
        None => Ok(None),
    }
}

pub async fn update_session_escrow(
    pool: &Pool<Sqlite>,
    id: &str,
    escrow_id: &str,
    buyer_address: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE payment_sessions SET status = 'funded', escrow_id = ?1, buyer_address = ?2 WHERE id = ?3",
    )
    .bind(escrow_id)
    .bind(buyer_address)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn complete_session(pool: &Pool<Sqlite>, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE payment_sessions SET status = 'completed' WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn list_sessions_by_app(
    pool: &Pool<Sqlite>,
    app_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<PaymentSession>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, app_id, escrow_id, amount_sompi, asset_type, seller_address, memo, status, buyer_address, created_at, expires_at, webhook_url, redirect_url
         FROM payment_sessions WHERE app_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
    )
    .bind(app_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let sessions = rows
        .into_iter()
        .map(|r| PaymentSession {
            id: r.try_get("id").unwrap_or_default(),
            app_id: r.try_get("app_id").unwrap_or_default(),
            escrow_id: r.try_get("escrow_id").unwrap_or(None),
            amount_sompi: r.try_get("amount_sompi").unwrap_or(0),
            asset_type: r.try_get("asset_type").unwrap_or_default(),
            seller_address: r.try_get("seller_address").unwrap_or_default(),
            memo: r.try_get("memo").unwrap_or(None),
            status: r.try_get("status").unwrap_or_default(),
            buyer_address: r.try_get("buyer_address").unwrap_or(None),
            created_at: r.try_get("created_at").unwrap_or(0),
            expires_at: r.try_get("expires_at").unwrap_or(0),
            webhook_url: r.try_get("webhook_url").unwrap_or(None),
            redirect_url: r.try_get("redirect_url").unwrap_or(None),
        })
        .collect();

    Ok(sessions)
}
