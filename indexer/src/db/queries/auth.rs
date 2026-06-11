use sqlx::{Pool, Row, Sqlite};

/// Store a used auth nonce to prevent replay attacks.
pub async fn store_auth_nonce(
    pool: &Pool<Sqlite>,
    nonce: &[u8],
    action: &str,
    escrow_id: &str,
    address: &str,
    created_at: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO auth_nonces (nonce, action, escrow_id, address, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(nonce)
    .bind(action)
    .bind(escrow_id)
    .bind(address)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Check if a nonce has already been used (for replay detection).
pub async fn check_auth_nonce_exists(
    pool: &Pool<Sqlite>,
    nonce: &[u8],
) -> Result<bool, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as cnt FROM auth_nonces WHERE nonce = ?1")
        .bind(nonce)
        .fetch_one(pool)
        .await?;
    let count: i64 = row.try_get("cnt").unwrap_or(0);
    Ok(count > 0)
}

/// Clean up expired auth nonces (older than cutoff timestamp).
pub async fn cleanup_expired_auth_nonces(
    pool: &Pool<Sqlite>,
    cutoff_timestamp: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM auth_nonces WHERE created_at < ?1")
        .bind(cutoff_timestamp)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}
