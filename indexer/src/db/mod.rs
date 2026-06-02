//! Database module for the DagLock indexer.
//!
//! Manages SQLite (dev) or PostgreSQL (prod) connections and migrations.
//! Provides query functions for escrows, offers, and reputation.

pub mod queries;
pub mod schema;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};
use std::str::FromStr;

/// Create a SQLite connection pool and run migrations.
pub async fn init_pool(database_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    // Run migrations
    schema::migrate(&pool).await?;

    Ok(pool)
}
