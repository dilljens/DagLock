//! Database schema migrations.

use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;
use std::collections::HashSet;

/// Run all pending migrations.
pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("migrations/001_create_escrows.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/002_create_offers.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/003_create_indexes.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/004_create_dispute_evidence.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!(
        "migrations/005_create_verified_identities.sql"
    ))
    .execute(pool)
    .await?;

    sqlx::query(include_str!("migrations/006_create_vouches.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/007_create_jury.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/008_create_escrow_messages.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/009_create_vaults.sql"))
        .execute(pool)
        .await?;

    ensure_offers_price_columns(pool).await?;
    ensure_escrow_trade_hash(pool).await?;
    ensure_escrow_market_order_fields(pool).await?;
    ensure_escrow_price_type(pool).await?;

    sqlx::query(include_str!("migrations/014_auth_nonces.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/016_create_apps.sql"))
        .execute(pool)
        .await?;

    ensure_escrow_lifecycle_columns(pool).await?;
    ensure_price_columns(pool).await?;
    ensure_price_type_column(pool).await?;
    ensure_dispute_mode_column(pool).await?;
    ensure_mediator_key_column(pool).await?;
    ensure_dispute_outcome_columns(pool).await?;
    ensure_lock_tx_id_index(pool).await?;

    Ok(())
}

async fn ensure_escrow_lifecycle_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("disputed_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN disputed_at INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("dispute_reason") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_reason TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("cancelled_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN cancelled_at INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("expired_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN expired_at INTEGER")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_price_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_at_creation") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_at_creation REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_currency") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_currency TEXT DEFAULT 'USD'")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_offers_price_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(offers)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_type TEXT DEFAULT 'fixed'")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_offset") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_offset REAL DEFAULT 0.0")
            .execute(pool)
            .await?;
    }
    if !existing.contains("min_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN min_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("max_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN max_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("current_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN current_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_currency") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_currency TEXT DEFAULT 'USD'")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_updated_at") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_updated_at INTEGER")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_trade_hash(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("trade_hash") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN trade_hash TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_market_order_fields(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_lock_time") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_lock_time INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_at_settlement") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_at_settlement REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_source") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_source TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_price_type(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_type TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_price_type_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_type TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_dispute_mode_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("dispute_mode") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_mode TEXT DEFAULT 'standard'")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_mediator_key_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("mediator_key") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediator_key TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_lock_tx_id_index(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Add unique index on lock_tx_id + lock_tx_output_index to prevent
    // duplicate escrows for the same UTXO
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_escrows_lock_tx
         ON escrows(lock_tx_id, lock_tx_output_index)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ensure_dispute_outcome_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("dispute_outcome") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_outcome TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("dispute_resolved_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_resolved_at INTEGER")
            .execute(pool)
            .await?;
    }

    Ok(())
}
