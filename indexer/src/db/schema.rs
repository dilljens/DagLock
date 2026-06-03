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

    ensure_escrow_lifecycle_columns(pool).await?;
    ensure_mediator_key_column(pool).await?;
    ensure_dispute_outcome_columns(pool).await?;

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
