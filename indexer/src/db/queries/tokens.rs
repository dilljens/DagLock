//! KRC-20 token aggregation queries.
//! Data is derived from existing offers and escrows — no separate token tracking needed.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRegistryEntry {
    pub id: String,
    pub ticker: String,
    pub name: String,
    pub total_supply: i64,
    pub decimals: i32,
    pub mint_mode: String,
    pub owner_address: Option<String>,
    pub covenant_address: Option<String>,
    pub template_hash: Option<Vec<u8>>,
    pub metadata_json: Option<String>,
    pub deploy_tx_id: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub deployed_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TokenSummary {
    pub ticker: String,
    pub price_sompi: Option<i64>,
    pub price_kas: Option<f64>,
    pub volume_24h_sompi: i64,
    pub trades_24h: i64,
    pub total_trades: i64,
    pub active_offers: i64,
    pub last_trade_at: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct TokenDetail {
    #[serde(flatten)]
    pub summary: TokenSummary,
    pub trades: Vec<TokenTrade>,
}

#[derive(Debug, Serialize)]
pub struct TokenTrade {
    pub escrow_id: String,
    pub amount_sompi: i64,
    pub price_sompi: Option<i64>,
    pub side: String,
    pub status: String,
    pub created_at: i64,
    pub buyer_address: String,
    pub seller_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenChartPoint {
    pub timestamp: i64,
    /// The KAS amount settled in trades at this time (not token price).
    /// Accurate price-per-token requires a token_amount field on escrows.
    pub volume_kas: f64,
}

/// List all KRC-20 tokens that have been traded or offered, with summary stats.
pub async fn list_tokens(pool: &Pool<Sqlite>) -> Result<Vec<TokenSummary>, sqlx::Error> {
    let now = Utc::now().timestamp();
    let since_24h = now - 86400;

    // Aggregate from offers (current asking prices) and escrows (trade history)
    let rows = sqlx::query(
        r#"
        SELECT
            COALESCE(o.quote_asset, e.asset_type) as ticker,
            AVG(o.amount_sompi) as avg_offer_price,
            COALESCE(SUM(CASE WHEN e.created_at >= ?2 THEN e.amount_sompi ELSE 0 END), 0) as volume_24h,
            COUNT(DISTINCT CASE WHEN e.created_at >= ?2 THEN e.id END) as trades_24h,
            COUNT(DISTINCT e.id) as total_trades,
            COUNT(DISTINCT o.id) as active_offers,
            MAX(e.created_at) as last_trade_at
        FROM (
            SELECT DISTINCT quote_asset as asset FROM offers WHERE status = 'proposed'
            UNION
            SELECT DISTINCT asset_type as asset FROM escrows
        ) assets
        LEFT JOIN offers o ON o.quote_asset = assets.asset AND o.status = 'proposed'
        LEFT JOIN escrows e ON e.asset_type = assets.asset
        WHERE assets.asset != 'KAS'
        GROUP BY assets.asset
        ORDER BY volume_24h DESC
        "#
    )
    .bind(now)
    .bind(since_24h)
    .fetch_all(pool)
    .await?;

    let tokens = rows
        .into_iter()
        .map(|row| {
            let ticker: String = row.get("ticker");
            let avg_price: Option<f64> = row.get("avg_offer_price");
            TokenSummary {
                ticker: ticker.trim_start_matches("KRC20:").to_string(),
                price_sompi: avg_price.map(|p| p as i64),
                price_kas: avg_price.map(|p| p / 100_000_000.0),
                volume_24h_sompi: row.get("volume_24h"),
                trades_24h: row.get("trades_24h"),
                total_trades: row.get("total_trades"),
                active_offers: row.get("active_offers"),
                last_trade_at: row.get("last_trade_at"),
            }
        })
        .collect();

    Ok(tokens)
}

/// Get detailed info for a specific KRC-20 token.
pub async fn get_token(
    pool: &Pool<Sqlite>,
    ticker: &str,
) -> Result<Option<TokenDetail>, sqlx::Error> {
    let asset = format!("KRC20:{ticker}");
    let now = Utc::now().timestamp();
    let since_24h = now - 86400;

    // Get summary
    let summary_row = sqlx::query(
        r#"
        SELECT
            AVG(o.amount_sompi) as avg_offer_price,
            COALESCE(SUM(CASE WHEN e.created_at >= ?2 THEN e.amount_sompi ELSE 0 END), 0) as volume_24h,
            COUNT(DISTINCT CASE WHEN e.created_at >= ?2 THEN e.id END) as trades_24h,
            COUNT(DISTINCT e.id) as total_trades,
            COUNT(DISTINCT o.id) as active_offers,
            MAX(e.created_at) as last_trade_at
        FROM (SELECT 1) dummy
        LEFT JOIN offers o ON o.quote_asset = ?1 AND o.status = 'proposed'
        LEFT JOIN escrows e ON e.asset_type = ?1
        "#
    )
    .bind(&asset)
    .bind(since_24h)
    .fetch_optional(pool)
    .await?;

    let summary_row = match summary_row {
        Some(r) => r,
        None => return Ok(None),
    };

    let avg_price: Option<f64> = summary_row.get("avg_offer_price");

    // Get recent trades
    let trade_rows = sqlx::query(
        r#"
        SELECT id, amount_sompi, status, created_at, buyer_address, seller_address
        FROM escrows
        WHERE asset_type = ?1
        ORDER BY created_at DESC
        LIMIT 20
        "#,
    )
    .bind(&asset)
    .fetch_all(pool)
    .await?;

    let trades = trade_rows
        .into_iter()
        .map(|row| TokenTrade {
            escrow_id: row.get("id"),
            amount_sompi: row.get("amount_sompi"),
            price_sompi: None,         // Price discovery from offers
            side: "trade".to_string(), // side depends on viewer's role — mark as generic
            status: row.get("status"),
            created_at: row.get("created_at"),
            buyer_address: row.get("buyer_address"),
            seller_address: row.get("seller_address"),
        })
        .collect();

    Ok(Some(TokenDetail {
        summary: TokenSummary {
            ticker: ticker.to_string(),
            price_sompi: avg_price.map(|p| p as i64),
            price_kas: avg_price.map(|p| p / 100_000_000.0),
            volume_24h_sompi: summary_row.get("volume_24h"),
            trades_24h: summary_row.get("trades_24h"),
            total_trades: summary_row.get("total_trades"),
            active_offers: summary_row.get("active_offers"),
            last_trade_at: summary_row.get("last_trade_at"),
        },
        trades,
    }))
}

/// Get price history for charting from escrow settlement data.
pub async fn get_token_chart(
    pool: &Pool<Sqlite>,
    ticker: &str,
    period_seconds: i64,
) -> Result<Vec<TokenChartPoint>, sqlx::Error> {
    let asset = format!("KRC20:{ticker}");
    let since = Utc::now().timestamp() - period_seconds;

    let rows = sqlx::query(
        r#"
        SELECT created_at, amount_sompi
        FROM escrows
        WHERE asset_type = ?1 AND created_at >= ?2 AND status = 'settled'
        ORDER BY created_at ASC
        "#,
    )
    .bind(&asset)
    .bind(since)
    .fetch_all(pool)
    .await?;

    let points = rows
        .into_iter()
        .map(|row| TokenChartPoint {
            timestamp: row.get("created_at"),
            volume_kas: row.get::<i64, _>("amount_sompi") as f64 / 100_000_000.0,
        })
        .collect();

    Ok(points)
}

/// ── Token Registry ─────────────────────────────────────────────────

pub async fn register_token(
    pool: &Pool<Sqlite>,
    id: &str,
    ticker: &str,
    name: &str,
    total_supply: i64,
    decimals: i32,
    mint_mode: &str,
    owner_address: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO token_registry \
         (id, ticker, name, total_supply, decimals, mint_mode, owner_address, status, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'pending', ?8)",
    )
    .bind(id)
    .bind(ticker.to_uppercase())
    .bind(name)
    .bind(total_supply)
    .bind(decimals)
    .bind(mint_mode)
    .bind(owner_address)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_registered_token(
    pool: &Pool<Sqlite>,
    ticker: &str,
) -> Result<Option<TokenRegistryEntry>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT id, ticker, name, total_supply, decimals, mint_mode, owner_address, \
         covenant_address, template_hash, metadata_json, deploy_tx_id, status, created_at, deployed_at \
         FROM token_registry WHERE ticker = ?1"
    )
    .bind(ticker.to_uppercase())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| TokenRegistryEntry {
        id: r.get("id"),
        ticker: r.get("ticker"),
        name: r.get("name"),
        total_supply: r.get("total_supply"),
        decimals: r.get("decimals"),
        mint_mode: r.get("mint_mode"),
        owner_address: r.get("owner_address"),
        covenant_address: r.get("covenant_address"),
        template_hash: r.get("template_hash"),
        metadata_json: r.get("metadata_json"),
        deploy_tx_id: r.get("deploy_tx_id"),
        status: r.get("status"),
        created_at: r.get("created_at"),
        deployed_at: r.get("deployed_at"),
    }))
}

pub async fn update_token_status(
    pool: &Pool<Sqlite>,
    ticker: &str,
    status: &str,
    covenant_address: Option<&str>,
    deploy_tx_id: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE token_registry SET status = ?1, covenant_address = COALESCE(?2, covenant_address), \
         deploy_tx_id = COALESCE(?3, deploy_tx_id), deployed_at = CASE WHEN ?1 = 'active' THEN ?4 ELSE deployed_at END \
         WHERE ticker = ?5"
    )
    .bind(status)
    .bind(covenant_address)
    .bind(deploy_tx_id)
    .bind(now)
    .bind(ticker.to_uppercase())
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_registered_tokens(
    pool: &Pool<Sqlite>,
    owner: Option<&str>,
) -> Result<Vec<TokenRegistryEntry>, sqlx::Error> {
    let rows = if let Some(addr) = owner {
        sqlx::query(
            "SELECT id, ticker, name, total_supply, decimals, mint_mode, owner_address, \
             covenant_address, template_hash, metadata_json, deploy_tx_id, status, created_at, deployed_at \
             FROM token_registry WHERE owner_address = ?1 ORDER BY created_at DESC"
        )
        .bind(addr)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            "SELECT id, ticker, name, total_supply, decimals, mint_mode, owner_address, \
             covenant_address, template_hash, metadata_json, deploy_tx_id, status, created_at, deployed_at \
             FROM token_registry ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await?
    };

    let entries = rows
        .into_iter()
        .map(|r| TokenRegistryEntry {
            id: r.get("id"),
            ticker: r.get("ticker"),
            name: r.get("name"),
            total_supply: r.get("total_supply"),
            decimals: r.get("decimals"),
            mint_mode: r.get("mint_mode"),
            owner_address: r.get("owner_address"),
            covenant_address: r.get("covenant_address"),
            template_hash: r.get("template_hash"),
            metadata_json: r.get("metadata_json"),
            deploy_tx_id: r.get("deploy_tx_id"),
            status: r.get("status"),
            created_at: r.get("created_at"),
            deployed_at: r.get("deployed_at"),
        })
        .collect();

    Ok(entries)
}
