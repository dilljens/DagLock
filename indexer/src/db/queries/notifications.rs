use sqlx::{Pool, Sqlite, Row};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct EmailSubscription {
    pub address: String,
    pub email: String,
    pub email_verified: bool,
    pub notify_created: bool,
    pub notify_settled: bool,
    pub notify_disputed: bool,
    pub notify_refunded: bool,
    pub notify_expired: bool,
}

pub async fn upsert_subscription(
    pool: &Pool<Sqlite>,
    address: &str,
    email: &str,
    verification_code: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO email_subscriptions (address, email, verification_code, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?4) \
         ON CONFLICT(address) DO UPDATE SET email = ?2, verification_code = ?3, email_verified = 0, updated_at = ?4"
    )
    .bind(address)
    .bind(email)
    .bind(verification_code)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn verify_email(
    pool: &Pool<Sqlite>,
    address: &str,
    code: &str,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE email_subscriptions SET email_verified = 1, verified_at = ?1, updated_at = ?1 \
         WHERE address = ?2 AND verification_code = ?3"
    )
    .bind(now)
    .bind(address)
    .bind(code)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_subscription(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Option<EmailSubscription>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT address, email, email_verified, notify_created, notify_settled, \
         notify_disputed, notify_refunded, notify_expired \
         FROM email_subscriptions WHERE address = ?1"
    )
    .bind(address)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| EmailSubscription {
        address: r.get("address"),
        email: r.get("email"),
        email_verified: r.get::<i64, _>("email_verified") != 0,
        notify_created: r.get::<i64, _>("notify_created") != 0,
        notify_settled: r.get::<i64, _>("notify_settled") != 0,
        notify_disputed: r.get::<i64, _>("notify_disputed") != 0,
        notify_refunded: r.get::<i64, _>("notify_refunded") != 0,
        notify_expired: r.get::<i64, _>("notify_expired") != 0,
    }))
}

pub async fn get_verified_subscribers_for_event(
    pool: &Pool<Sqlite>,
    event_column: &str,
) -> Result<Vec<EmailSubscription>, sqlx::Error> {
    let rows = sqlx::query(
        &format!(
            "SELECT address, email, email_verified, notify_created, notify_settled, \
             notify_disputed, notify_refunded, notify_expired \
             FROM email_subscriptions WHERE email_verified = 1 AND {} = 1",
            event_column
        )
    )
    .fetch_all(pool)
    .await?;

    let subs = rows.into_iter().map(|r| EmailSubscription {
        address: r.get("address"),
        email: r.get("email"),
        email_verified: r.get::<i64, _>("email_verified") != 0,
        notify_created: r.get::<i64, _>("notify_created") != 0,
        notify_settled: r.get::<i64, _>("notify_settled") != 0,
        notify_disputed: r.get::<i64, _>("notify_disputed") != 0,
        notify_refunded: r.get::<i64, _>("notify_refunded") != 0,
        notify_expired: r.get::<i64, _>("notify_expired") != 0,
    }).collect();

    Ok(subs)
}

pub async fn update_preferences(
    pool: &Pool<Sqlite>,
    address: &str,
    notify_created: Option<bool>,
    notify_settled: Option<bool>,
    notify_disputed: Option<bool>,
    notify_refunded: Option<bool>,
    notify_expired: Option<bool>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "UPDATE email_subscriptions SET \
         notify_created = COALESCE(?1, notify_created), \
         notify_settled = COALESCE(?2, notify_settled), \
         notify_disputed = COALESCE(?3, notify_disputed), \
         notify_refunded = COALESCE(?4, notify_refunded), \
         notify_expired = COALESCE(?5, notify_expired), \
         updated_at = ?6 WHERE address = ?7"
    )
    .bind(notify_created.map(|v| v as i64))
    .bind(notify_settled.map(|v| v as i64))
    .bind(notify_disputed.map(|v| v as i64))
    .bind(notify_refunded.map(|v| v as i64))
    .bind(notify_expired.map(|v| v as i64))
    .bind(now)
    .bind(address)
    .execute(pool)
    .await?;
    Ok(())
}
