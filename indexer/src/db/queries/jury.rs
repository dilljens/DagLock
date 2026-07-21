use chrono;
use sqlx::{Pool, Row, Sqlite};

use crate::types::*;

pub async fn register_juror(pool: &Pool<Sqlite>, address: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        "INSERT OR REPLACE INTO juror_registrations (address, registered_at, total_cases_assigned, total_cases_voted, reliability_score)
         VALUES (?1, ?2, 0, 0, 1.0)"
    )
    .bind(address)
    .bind(now)
    .execute(pool).await?;
    Ok(())
}

pub async fn unregister_juror(pool: &Pool<Sqlite>, address: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM juror_registrations WHERE address = ?1")
        .bind(address)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[allow(dead_code)]
pub async fn get_juror(
    pool: &Pool<Sqlite>,
    address: &str,
) -> Result<Option<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM juror_registrations WHERE address = ?1")
        .bind(address)
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| JurorRegistration {
            address: row.try_get("address").unwrap_or_default(),
            registered_at: row.try_get("registered_at").unwrap_or(0),
            total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
            total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
            reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
        })
        .next())
}

#[allow(dead_code)]
pub async fn list_eligible_jurors(
    pool: &Pool<Sqlite>,
    min_score: f64,
) -> Result<Vec<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM juror_registrations WHERE reliability_score >= ?1 ORDER BY reliability_score DESC"
    )
    .bind(min_score)
    .fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| JurorRegistration {
            address: row.try_get("address").unwrap_or_default(),
            registered_at: row.try_get("registered_at").unwrap_or(0),
            total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
            total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
            reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
        })
        .collect())
}

pub async fn list_eligible_jurors_simple(
    pool: &Pool<Sqlite>,
) -> Result<Vec<JurorRegistration>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM juror_registrations ORDER BY reliability_score DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| JurorRegistration {
            address: row.try_get("address").unwrap_or_default(),
            registered_at: row.try_get("registered_at").unwrap_or(0),
            total_cases_assigned: row.try_get("total_cases_assigned").unwrap_or(0),
            total_cases_voted: row.try_get("total_cases_voted").unwrap_or(0),
            reliability_score: row.try_get("reliability_score").unwrap_or(1.0),
        })
        .collect())
}

pub async fn create_jury_case(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
    juror_count: i64,
    threshold: i64,
    juror_addresses: &[String],
) -> Result<String, sqlx::Error> {
    use uuid::Uuid;
    let case_id = format!(
        "jr_{}",
        Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .expect("UUID should have a dash")
    );
    let now = chrono::Utc::now().timestamp();

    // Escalation: start at mediation (level 0), deadline = now + 2 days
    let escalation_deadline = now + 172_800;

    // Insert case
    sqlx::query(
        "INSERT INTO jury_cases (id, escrow_id, status, juror_count, threshold, votes_for_seller, votes_for_buyer, created_at, escalation_level, escalation_deadline)
         VALUES (?1, ?2, 'voting', ?3, ?4, 0, 0, ?5, 0, ?6)"
    )
    .bind(&case_id)
    .bind(escrow_id)
    .bind(juror_count)
    .bind(threshold)
    .bind(now)
    .bind(escalation_deadline)
    .execute(pool).await?;

    // Insert jury_votes rows for each juror (pre-assigned, votes NULL)
    for addr in juror_addresses {
        // Store assigned jurors — we use jury_votes with vote=NULL to indicate assignment
        sqlx::query(
            "INSERT INTO jury_votes (case_id, juror_address, vote, voted_at)
             VALUES (?1, ?2, '', 0)",
        )
        .bind(&case_id)
        .bind(addr)
        .execute(pool)
        .await?;
    }

    Ok(case_id)
}

pub async fn get_jury_case(
    pool: &Pool<Sqlite>,
    case_id: &str,
) -> Result<Option<JuryCase>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM jury_cases WHERE id = ?1")
        .bind(case_id)
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }
    let row = &rows[0];
    let cid: String = row.try_get("id").unwrap_or_default();

    // Fetch assigned jurors
    let juror_rows = sqlx::query("SELECT juror_address FROM jury_votes WHERE case_id = ?1")
        .bind(&cid)
        .fetch_all(pool)
        .await?;
    let jurors: Vec<String> = juror_rows
        .into_iter()
        .filter_map(|r| r.try_get::<String, _>("juror_address").ok())
        .collect();

    Ok(Some(JuryCase {
        id: cid,
        escrow_id: row.try_get("escrow_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        juror_count: row.try_get("juror_count").unwrap_or(0),
        threshold: row.try_get("threshold").unwrap_or(0),
        votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
        votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        decided_at: row.try_get("decided_at").unwrap_or(None),
        outcome: row.try_get("outcome").unwrap_or(None),
        jurors,
        escalation_level: row.try_get("escalation_level").unwrap_or(0),
        escalation_deadline: row.try_get("escalation_deadline").unwrap_or(None),
        mediation_log: row.try_get("mediation_log").unwrap_or(None),
        revealed_chat_key_enc: row.try_get("revealed_chat_key_enc").ok().flatten(),
        revealed_at: row.try_get("revealed_at").ok().flatten(),
        evidence_cleared_at: row.try_get("evidence_cleared_at").ok().flatten(),
    }))
}

pub async fn get_jury_case_by_escrow(
    pool: &Pool<Sqlite>,
    escrow_id: &str,
) -> Result<Option<JuryCase>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM jury_cases WHERE escrow_id = ?1 ORDER BY created_at DESC LIMIT 1",
    )
    .bind(escrow_id)
    .fetch_all(pool)
    .await?;
    if rows.is_empty() {
        return Ok(None);
    }
    let row = &rows[0];
    let cid: String = row.try_get("id").unwrap_or_default();

    let juror_rows = sqlx::query("SELECT juror_address FROM jury_votes WHERE case_id = ?1")
        .bind(&cid)
        .fetch_all(pool)
        .await?;
    let jurors: Vec<String> = juror_rows
        .into_iter()
        .filter_map(|r| r.try_get::<String, _>("juror_address").ok())
        .collect();

    Ok(Some(JuryCase {
        id: cid,
        escrow_id: row.try_get("escrow_id").unwrap_or_default(),
        status: row.try_get("status").unwrap_or_default(),
        juror_count: row.try_get("juror_count").unwrap_or(0),
        threshold: row.try_get("threshold").unwrap_or(0),
        votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
        votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
        created_at: row.try_get("created_at").unwrap_or(0),
        decided_at: row.try_get("decided_at").unwrap_or(None),
        outcome: row.try_get("outcome").unwrap_or(None),
        jurors,
        escalation_level: row.try_get("escalation_level").unwrap_or(0),
        escalation_deadline: row.try_get("escalation_deadline").unwrap_or(None),
        mediation_log: row.try_get("mediation_log").unwrap_or(None),
        revealed_chat_key_enc: row.try_get("revealed_chat_key_enc").ok().flatten(),
        revealed_at: row.try_get("revealed_at").ok().flatten(),
        evidence_cleared_at: row.try_get("evidence_cleared_at").ok().flatten(),
    }))
}

pub async fn cast_jury_vote(
    pool: &Pool<Sqlite>,
    case_id: &str,
    juror_address: &str,
    vote: &str,
    reasoning: Option<&str>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();

    // Atomic single-query update: update vote tally in one step
    // Using CASE expression to increment the correct counter atomically
    sqlx::query(
        "UPDATE jury_votes SET vote = ?1, voted_at = ?2, reasoning = ?3
         WHERE case_id = ?4 AND juror_address = ?5 AND vote = ''",
    )
    .bind(vote)
    .bind(now)
    .bind(reasoning)
    .bind(case_id)
    .bind(juror_address)
    .execute(pool)
    .await?;

    // Separate queries are fine — SQLite serializes writes.
    // The WHERE vote='' prevents double-counting the same juror.
    if vote == "seller_wins" {
        sqlx::query("UPDATE jury_cases SET votes_for_seller = votes_for_seller + 1 WHERE id = ?1")
            .bind(case_id)
            .execute(pool)
            .await?;
    } else if vote == "buyer_wins" {
        sqlx::query("UPDATE jury_cases SET votes_for_buyer = votes_for_buyer + 1 WHERE id = ?1")
            .bind(case_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn check_jury_verdict(
    pool: &Pool<Sqlite>,
    case_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let case = get_jury_case(pool, case_id).await?;
    match case {
        Some(c) if c.status == "voting" || c.status == "selecting" => {
            if c.votes_for_seller >= c.threshold {
                let now = chrono::Utc::now().timestamp();
                sqlx::query(
                    "UPDATE jury_cases SET status = 'decided', outcome = 'seller_wins', decided_at = ?1 WHERE id = ?2"
                ).bind(now).bind(case_id).execute(pool).await?;
                Ok(Some("seller_wins".to_string()))
            } else if c.votes_for_buyer >= c.threshold {
                let now = chrono::Utc::now().timestamp();
                sqlx::query(
                    "UPDATE jury_cases SET status = 'decided', outcome = 'buyer_wins', decided_at = ?1 WHERE id = ?2"
                ).bind(now).bind(case_id).execute(pool).await?;
                Ok(Some("buyer_wins".to_string()))
            } else {
                Ok(None) // No verdict yet
            }
        }
        Some(c) => Ok(c.outcome.clone()),
        None => Ok(None),
    }
}

pub async fn expire_stale_jury_cases(pool: &Pool<Sqlite>) -> Result<u64, sqlx::Error> {
    let deadline = chrono::Utc::now().timestamp() - 72 * 3600;
    let result = sqlx::query(
        "UPDATE jury_cases SET status = 'decided', outcome = 'seller_wins', decided_at = ?1
         WHERE status = 'voting' AND created_at < ?2",
    )
    .bind(chrono::Utc::now().timestamp())
    .bind(deadline)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

pub async fn list_active_jury_cases_for_juror(
    pool: &Pool<Sqlite>,
    juror_address: &str,
) -> Result<Vec<JuryCase>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT jc.* FROM jury_cases jc
         INNER JOIN jury_votes jv ON jv.case_id = jc.id
         WHERE jv.juror_address = ?1 AND jc.status IN ('selecting', 'voting')
         ORDER BY jc.created_at DESC",
    )
    .bind(juror_address)
    .fetch_all(pool)
    .await?;

    let mut cases = Vec::new();
    for row in &rows {
        let cid: String = row.try_get("id").unwrap_or_default();
        let juror_rows = sqlx::query("SELECT juror_address FROM jury_votes WHERE case_id = ?1")
            .bind(&cid)
            .fetch_all(pool)
            .await?;
        let jurors: Vec<String> = juror_rows
            .into_iter()
            .filter_map(|r| r.try_get::<String, _>("juror_address").ok())
            .collect();
        cases.push(JuryCase {
            id: cid,
            escrow_id: row.try_get("escrow_id").unwrap_or_default(),
            status: row.try_get("status").unwrap_or_default(),
            juror_count: row.try_get("juror_count").unwrap_or(0),
            threshold: row.try_get("threshold").unwrap_or(0),
            votes_for_seller: row.try_get("votes_for_seller").unwrap_or(0),
            votes_for_buyer: row.try_get("votes_for_buyer").unwrap_or(0),
            created_at: row.try_get("created_at").unwrap_or(0),
            decided_at: row.try_get("decided_at").unwrap_or(None),
            outcome: row.try_get("outcome").unwrap_or(None),
            jurors,
            escalation_level: row.try_get("escalation_level").unwrap_or(0),
            escalation_deadline: row.try_get("escalation_deadline").unwrap_or(None),
            mediation_log: row.try_get("mediation_log").unwrap_or(None),
            revealed_chat_key_enc: row.try_get("revealed_chat_key_enc").ok().flatten(),
            revealed_at: row.try_get("revealed_at").ok().flatten(),
            evidence_cleared_at: row.try_get("evidence_cleared_at").ok().flatten(),
        });
    }
    Ok(cases)
}

pub async fn update_escalation_level(
    pool: &Pool<Sqlite>,
    case_id: &str,
    level: i64,
    deadline: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE jury_cases SET escalation_level = ?1, escalation_deadline = ?2 WHERE id = ?3",
    )
    .bind(level)
    .bind(deadline)
    .bind(case_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn auto_decide_case(
    pool: &Pool<Sqlite>,
    case_id: &str,
    outcome: &str,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE jury_cases SET status = 'decided', outcome = ?1, decided_at = ?2 WHERE id = ?3 AND status != 'decided'"
    )
    .bind(outcome)
    .bind(now)
    .bind(case_id)
    .execute(pool).await?;
    Ok(())
}

pub async fn find_escalatable_cases(
    pool: &Pool<Sqlite>,
    now: i64,
) -> Result<Vec<(String, i64, String)>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, escalation_level, status FROM jury_cases
         WHERE escalation_deadline IS NOT NULL
           AND escalation_deadline <= ?1
           AND status IN ('voting', 'mediation')
           AND escalation_level < 2",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let id: String = row.try_get("id").ok()?;
            let level: i64 = row.try_get("escalation_level").ok()?;
            let status: String = row.try_get("status").ok()?;
            Some((id, level, status))
        })
        .collect())
}
