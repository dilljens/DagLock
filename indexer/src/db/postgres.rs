//! PostgreSQL database support.
//!
//! Provides PostgreSQL-specific migrations and connection handling.
//! Use this when deploying to production with PostgreSQL.

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

/// Create a PostgreSQL connection pool.
pub async fn create_postgres_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    info!("Connected to PostgreSQL: {}", database_url);
    Ok(pool)
}

/// Run PostgreSQL migrations.
pub async fn migrate_postgres(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("migrations_pg/001_create_escrows.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations_pg/002_create_offers.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations_pg/003_create_indexes.sql"))
        .execute(pool)
        .await?;

    info!("PostgreSQL migrations completed");
    Ok(())
}
