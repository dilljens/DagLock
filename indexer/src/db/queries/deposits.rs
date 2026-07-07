use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::Deposit;

pub async fn insert_deposit(pool: &Pool<Sqlite>, deposit: &Deposit) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO deposits (id, escrow_id, party1_address, party2_address, deposit_amount, status, deposit_tx_id, timeout, created_at, released_at, forfeited_at, forfeited_to)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )
    .bind(&deposit.id)
    .bind(&deposit.escrow_id)
    .bind(&deposit.party1_address)
    .bind(&deposit.party2_address)
    .bind(deposit.deposit_amount)
    .bind(&deposit.status)
    .bind(&deposit.deposit_tx_id)
    .bind(deposit.timeout)
    .bind(deposit.created_at)
    .bind(deposit.released_at)
    .bind(deposit.forfeited_at)
    .bind(&deposit.forfeited_to)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_deposit(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Deposit>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM deposits WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(row_to_deposit))
}

pub async fn get_deposit_by_escrow(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Option<Deposit>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM deposits WHERE escrow_id = ?1")
        .bind(escrow_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(row_to_deposit))
}

pub async fn release_deposit(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE deposits SET status = 'released', released_at = ?1 WHERE id = ?2 AND status = 'locked'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn forfeit_deposit(
    pool: &Pool<Sqlite>,
    id: &str,
    forfeited_to: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE deposits SET status = 'forfeited', forfeited_at = ?1, forfeited_to = ?2 WHERE id = ?3 AND status = 'locked'",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(forfeited_to)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn sweep_deposit(
    pool: &Pool<Sqlite>,
    id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE deposits SET status = 'swept' WHERE id = ?1 AND status = 'locked'",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_stale_deposits(
    pool: &Pool<Sqlite>,
) -> Result<Vec<Deposit>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let rows = sqlx::query(
        "SELECT * FROM deposits WHERE status = 'locked' AND timeout > 0 AND timeout <= ?1",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_deposit).collect())
}

fn row_to_deposit(row: sqlx::sqlite::SqliteRow) -> Deposit {
    Deposit {
        id: row.try_get("id").unwrap_or_default(),
        escrow_id: row.try_get("escrow_id").unwrap_or_default(),
        party1_address: row.try_get("party1_address").unwrap_or_default(),
        party2_address: row.try_get("party2_address").unwrap_or_default(),
        deposit_amount: row.try_get("deposit_amount").unwrap_or(0),
        status: row.try_get("status").unwrap_or_else(|_| "locked".to_string()),
        deposit_tx_id: row.try_get("deposit_tx_id").ok().flatten(),
        timeout: row.try_get("timeout").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        released_at: row.try_get("released_at").ok().flatten(),
        forfeited_at: row.try_get("forfeited_at").ok().flatten(),
        forfeited_to: row.try_get("forfeited_to").ok().flatten(),
    }
}
