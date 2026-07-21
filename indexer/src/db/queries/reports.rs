use serde::Serialize;
use sqlx::{Pool, Row, Sqlite};

#[derive(Serialize)]
pub struct UserReport {
    pub id: String,
    pub reporter_address: String,
    pub reported_address: String,
    pub escrow_id: Option<String>,
    pub reason: String,
    pub created_at: i64,
}

pub async fn create_report(
    pool: &Pool<Sqlite>,
    id: &str,
    reporter: &str,
    reported: &str,
    escrow_id: Option<&str>,
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO user_reports (id, reporter_address, reported_address, escrow_id, reason, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )
    .bind(id)
    .bind(reporter)
    .bind(reported)
    .bind(escrow_id)
    .bind(reason)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_all_reports(
    pool: &Pool<Sqlite>,
    limit: i64,
    offset: i64,
) -> Result<Vec<UserReport>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, i64)>(
        "SELECT id, reporter_address, reported_address, escrow_id, reason, created_at FROM user_reports ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let reports = rows
        .into_iter()
        .map(|r| UserReport {
            id: r.0,
            reporter_address: r.1,
            reported_address: r.2,
            escrow_id: r.3,
            reason: r.4,
            created_at: r.5,
        })
        .collect();

    Ok(reports)
}

pub async fn list_reports(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Vec<UserReport>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, reporter_address, reported_address, escrow_id, reason, created_at FROM user_reports WHERE reported_address = ?1 ORDER BY created_at DESC"
    )
    .bind(address)
    .fetch_all(pool)
    .await?;

    let reports = rows
        .into_iter()
        .map(|row| UserReport {
            id: row.get("id"),
            reporter_address: row.get("reporter_address"),
            reported_address: row.get("reported_address"),
            escrow_id: row.get("escrow_id"),
            reason: row.get("reason"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(reports)
}
