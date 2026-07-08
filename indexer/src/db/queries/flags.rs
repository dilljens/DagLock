//! Account flags — per-address metadata (is_bot, label, etc.).
//!
//! Used by the reputation system and offer creation to distinguish
//! bot accounts from human accounts.

use chrono;
use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;

use crate::types::{AccountFlags, SetAccountFlagsRequest};

/// Upsert account flags for an address.
/// INSERT OR REPLACE so calling it multiple times is safe.
pub async fn upsert_account_flags(
    pool: &Pool<Sqlite>,
    req: &SetAccountFlagsRequest,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR REPLACE INTO account_flags (address, is_bot, label, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(&req.address)
    .bind(req.is_bot as i32)
    .bind(&req.label)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get account flags for an address. Returns None if no flags set.
pub async fn get_account_flags(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Option<AccountFlags>, sqlx::Error> {
    let row = sqlx::query("SELECT address, is_bot, label, updated_at FROM account_flags WHERE address = ?1")
        .bind(address)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => Ok(Some(AccountFlags {
            address: r.try_get("address")?,
            is_bot: r.try_get::<i32, _>("is_bot")? != 0,
            label: r.try_get("label").ok().flatten(),
            updated_at: r.try_get("updated_at")?,
        })),
        None => Ok(None),
    }
}

/// Check if an address is flagged as a bot.
pub async fn is_bot_address(pool: &Pool<Sqlite>, address: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT is_bot FROM account_flags WHERE address = ?1")
        .bind(address)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => Ok(r.try_get::<i32, _>("is_bot")? != 0),
        None => Ok(false),
    }
}
