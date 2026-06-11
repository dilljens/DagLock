use chrono;
use sqlx::{Pool, Row, Sqlite};

pub async fn upsert_identity(
    pool: &Pool<Sqlite>,
    address: &str,
    platform: &str,
    handle: &str,
    signed_message: &str,
    signature_hex: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR REPLACE INTO verified_identities (address, platform, handle, signed_message, signature_hex, verified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind(address)
    .bind(platform)
    .bind(handle)
    .bind(signed_message)
    .bind(signature_hex)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_identity_handle(
    pool: &Pool<Sqlite>,
    address: &str,
    platform: &str,
) -> Result<Option<String>, sqlx::Error> {
    let rows =
        sqlx::query("SELECT handle FROM verified_identities WHERE address = ?1 AND platform = ?2")
            .bind(address)
            .bind(platform)
            .fetch_all(pool)
            .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| r.try_get::<String, _>("handle").ok())
        .next())
}
