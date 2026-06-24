use blake2b_simd::Params;
use chrono;
use sqlx::{Pool, Row, Sqlite};

/// Register a new app and generate an API key.
/// Returns the app record + the plaintext API key.
pub async fn register_app(
    pool: &Pool<Sqlite>,
    name: &str,
    callback_url: Option<&str>,
    owner_address: &str,
) -> Result<(crate::types::App, String), sqlx::Error> {
    let app_id = format!(
        "app_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    let now = chrono::Utc::now().timestamp();

    // Generate a secure API key and hash it
    let key_plaintext = format!(
        "dl_sk_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    let key_hash = Params::new()
        .hash_length(32)
        .hash(key_plaintext.as_bytes())
        .as_bytes()
        .to_vec();

    // Insert app
    sqlx::query(
        "INSERT INTO apps (id, name, callback_url, created_at, owner_address, is_active)
         VALUES (?1, ?2, ?3, ?4, ?5, 1)",
    )
    .bind(&app_id)
    .bind(name)
    .bind(callback_url)
    .bind(now)
    .bind(owner_address)
    .execute(pool)
    .await?;

    // Insert API key hash
    let key_id = format!(
        "k_{}",
        uuid::Uuid::new_v4().to_string().replace('-', "")
    );
    sqlx::query(
        "INSERT INTO api_keys (id, key_hash, app_id, label, created_at, is_active)
         VALUES (?1, ?2, ?3, 'default', ?4, 1)",
    )
    .bind(&key_id)
    .bind(&key_hash)
    .bind(&app_id)
    .bind(now)
    .execute(pool)
    .await?;

    let app = crate::types::App {
        id: app_id,
        name: name.to_string(),
        callback_url: callback_url.map(|s| s.to_string()),
        webhook_secret: None,
        created_at: now,
        owner_address: owner_address.to_string(),
        is_active: true,
    };

    Ok((app, key_plaintext))
}

/// Look up an app by API key (hashed). Returns the app if the key is valid and active.
pub async fn verify_api_key(
    pool: &Pool<Sqlite>,
    api_key: &str,
) -> Result<Option<crate::types::App>, sqlx::Error> {
    let key_hash = Params::new()
        .hash_length(32)
        .hash(api_key.as_bytes())
        .as_bytes()
        .to_vec();

    let row = sqlx::query(
        "SELECT a.id AS app_id_field, a.name, a.callback_url, a.webhook_secret, a.created_at, a.owner_address, a.is_active
         FROM apps a
         INNER JOIN api_keys k ON k.app_id = a.id
         WHERE k.key_hash = ?1 AND k.is_active = 1 AND a.is_active = 1"
    )
    .bind(&key_hash)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| crate::types::App {
        id: r.try_get("id").unwrap_or_default(),
        name: r.try_get("name").unwrap_or_default(),
        callback_url: r.try_get("callback_url").ok().flatten(),
        webhook_secret: r.try_get("webhook_secret").ok().flatten(),
        created_at: r.try_get("created_at").unwrap_or(0),
        owner_address: r.try_get("owner_address").unwrap_or_default(),
        is_active: r.try_get("is_active").unwrap_or(1) != 0,
    }))
}

/// Update last_used_at for an API key.
pub async fn touch_api_key(pool: &Pool<Sqlite>, api_key: &str) -> Result<(), sqlx::Error> {
    let key_hash = Params::new()
        .hash_length(32)
        .hash(api_key.as_bytes())
        .as_bytes()
        .to_vec();
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE api_keys SET last_used_at = ?1 WHERE key_hash = ?2")
        .bind(now)
        .bind(&key_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get an app by ID.
pub async fn get_app(
    pool: &Pool<Sqlite>,
    app_id: &str,
) -> Result<Option<crate::types::App>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM apps WHERE id = ?1")
        .bind(app_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(row_to_app).next())
}

/// List API keys for an app.
pub async fn list_api_keys(
    pool: &Pool<Sqlite>,
    app_id: &str,
) -> Result<Vec<crate::types::ApiKey>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT k.id, k.app_id, k.label, k.created_at, k.last_used_at, k.is_active
         FROM api_keys k WHERE k.app_id = ?1 ORDER BY k.created_at DESC",
    )
    .bind(app_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(row_to_api_key).collect())
}

/// Revoke an API key (soft delete).
pub async fn revoke_api_key(
    pool: &Pool<Sqlite>,
    key_id: &str,
    app_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE api_keys SET is_active = 0 WHERE id = ?1 AND app_id = ?2")
        .bind(key_id)
        .bind(app_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

fn row_to_app(row: sqlx::sqlite::SqliteRow) -> crate::types::App {
    crate::types::App {
        id: row.try_get("id").unwrap_or_default(),
        name: row.try_get("name").unwrap_or_default(),
        callback_url: row.try_get("callback_url").ok().flatten(),
        webhook_secret: row.try_get("webhook_secret").ok().flatten(),
        created_at: row.try_get("created_at").unwrap_or(0),
        owner_address: row.try_get("owner_address").unwrap_or_default(),
        is_active: row.try_get("is_active").unwrap_or(1) != 0,
    }
}

fn row_to_api_key(row: sqlx::sqlite::SqliteRow) -> crate::types::ApiKey {
    crate::types::ApiKey {
        key_id: row.try_get("id").unwrap_or_default(),
        app_id: row.try_get("app_id").unwrap_or_default(),
        label: row.try_get("label").unwrap_or_default(),
        created_at: row.try_get("created_at").unwrap_or(0),
        last_used_at: row.try_get("last_used_at").ok().flatten(),
        is_active: row.try_get("is_active").unwrap_or(1) != 0,
    }
}
