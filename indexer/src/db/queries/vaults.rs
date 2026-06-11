use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn insert_vault(pool: &Pool<Sqlite>, vault: &Vault) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO vaults (id, owner_address, beneficiary_address, vault_type, status, amount_sompi, timeout, lock_tx_id, lock_tx_output_index, created_at, unlocked_at, expires_at, owner_pubkey_hex, sweep_tx_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
    )
    .bind(&vault.id)
    .bind(&vault.owner_address)
    .bind(&vault.beneficiary_address)
    .bind(serde_json::to_string(&vault.vault_type).unwrap_or_else(|_| "\"time\"".to_string()))
    .bind(serde_json::to_string(&vault.status).unwrap_or_else(|_| "\"locked\"".to_string()))
    .bind(vault.amount_sompi)
    .bind(vault.timeout)
    .bind(&vault.lock_tx_id)
    .bind(vault.lock_tx_output_index)
    .bind(vault.created_at)
    .bind(vault.unlocked_at)
    .bind(vault.expires_at)
    .bind(&vault.owner_pubkey_hex)
    .bind(&vault.sweep_tx_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_vault(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Vault>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM vaults WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(row_to_vault))
}

pub async fn list_vaults_by_owner(
    pool: &Pool<Sqlite>,
    owner: &str,
) -> Result<Vec<Vault>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT * FROM vaults WHERE owner_address = ?1 ORDER BY created_at DESC")
            .bind(owner)
            .fetch_all(pool)
            .await?;

    Ok(rows.into_iter().map(row_to_vault).collect())
}

pub async fn count_vaults_by_owner(pool: &Pool<Sqlite>, owner: &str) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM vaults WHERE owner_address = ?1")
        .bind(owner)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

pub async fn update_vault_status(
    pool: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE vaults SET status = ?1 WHERE id = ?2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_vault_beneficiary(
    pool: &Pool<Sqlite>,
    id: &str,
    beneficiary: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE vaults SET beneficiary_address = ?1 WHERE id = ?2")
        .bind(beneficiary)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn find_sweepable_vaults(
    pool: &Pool<Sqlite>,
) -> Result<Vec<(String, String, i64, Option<String>)>, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query_as::<_, (String, String, i64, Option<String>)>(
        "SELECT id, owner_address, amount_sompi, owner_pubkey_hex FROM vaults WHERE status = 'locked' AND timeout < ?1 AND sweep_tx_id IS NULL",
    )
    .bind(now)
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn mark_vault_swept(
    pool: &Pool<Sqlite>,
    id: &str,
    sweep_tx_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE vaults SET status = 'unlocked', sweep_tx_id = ?1, unlocked_at = ?2 WHERE id = ?3")
        .bind(sweep_tx_id)
        .bind(chrono::Utc::now().timestamp())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

fn row_to_vault(row: sqlx::sqlite::SqliteRow) -> Vault {
    Vault {
        id: row.try_get("id").unwrap_or_default(),
        owner_address: row.try_get("owner_address").unwrap_or_default(),
        beneficiary_address: row.try_get("beneficiary_address").ok().flatten(),
        vault_type: serde_json::from_str(
            &row.try_get::<String, _>("vault_type")
                .unwrap_or_else(|_| "\"time\"".to_string()),
        )
        .unwrap_or(VaultType::Time),
        status: serde_json::from_str(
            &row.try_get::<String, _>("status")
                .unwrap_or_else(|_| "\"locked\"".to_string()),
        )
        .unwrap_or(VaultStatus::Locked),
        amount_sompi: row.try_get("amount_sompi").unwrap_or(0),
        timeout: row.try_get("timeout").unwrap_or(0),
        lock_tx_id: row.try_get("lock_tx_id").ok().flatten(),
        lock_tx_output_index: row.try_get("lock_tx_output_index").ok().flatten(),
        created_at: row.try_get("created_at").unwrap_or(0),
        unlocked_at: row.try_get("unlocked_at").ok().flatten(),
        expires_at: row.try_get("expires_at").ok().flatten(),
        owner_pubkey_hex: row.try_get("owner_pubkey_hex").ok().flatten(),
        sweep_tx_id: row.try_get("sweep_tx_id").ok().flatten(),
    }
}
