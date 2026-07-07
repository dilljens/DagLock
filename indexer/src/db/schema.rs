//! Database schema migrations.

use sqlx::Pool;
use sqlx::Row;
use sqlx::Sqlite;
use std::collections::HashSet;

/// Run all pending migrations.
pub async fn migrate(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(include_str!("migrations/001_create_escrows.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/002_create_offers.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/003_create_indexes.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/004_create_dispute_evidence.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!(
        "migrations/005_create_verified_identities.sql"
    ))
    .execute(pool)
    .await?;

    sqlx::query(include_str!("migrations/006_create_vouches.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/007_create_jury.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/008_create_escrow_messages.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/009_create_vaults.sql"))
        .execute(pool)
        .await?;

    ensure_offers_price_columns(pool).await?;
    ensure_escrow_trade_hash(pool).await?;
    ensure_escrow_market_order_fields(pool).await?;
    ensure_escrow_price_type(pool).await?;

    sqlx::query(include_str!("migrations/014_auth_nonces.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/016_create_apps.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/017_create_webhooks.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/018_create_invoices.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/019_create_blocked_users.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/020_create_reports.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/021_create_trade_feedback.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/022_create_counteroffers.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/025_token_registry.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/026_email_subscriptions.sql"))
        .execute(pool)
        .await?;

    sqlx::query(include_str!("migrations/027_create_subscriptions.sql"))
        .execute(pool)
        .await?;

    ensure_escrow_lifecycle_columns(pool).await?;
    ensure_escrow_invoice_id_column(pool).await?;
    ensure_escrow_memo_column(pool).await?;
    ensure_price_columns(pool).await?;
    ensure_price_type_column(pool).await?;
    ensure_dispute_mode_column(pool).await?;
    ensure_mediator_key_column(pool).await?;
    ensure_dispute_outcome_columns(pool).await?;
    ensure_lock_tx_id_index(pool).await?;
    ensure_vault_sweep_columns(pool).await?;
    ensure_auto_settle_column(pool).await?;
    ensure_dispute_escalation_columns(pool).await?;
    ensure_milestone_escrows_table(pool).await?;
    ensure_multi_escrows_table(pool).await?;
    ensure_deposits_table(pool).await?;
    ensure_mediation_columns(pool).await?;
    ensure_chat_columns(pool).await?;

    Ok(())
}

pub async fn ensure_multi_escrows_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(multi_escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("id") {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS multi_escrows (
                id TEXT PRIMARY KEY,
                lock_tx_id TEXT NOT NULL,
                parties TEXT NOT NULL,
                shares TEXT NOT NULL,
                total_amount INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at INTEGER NOT NULL,
                settled_at INTEGER,
                refunded_at INTEGER,
                signatures TEXT NOT NULL DEFAULT '[]'
            )"
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn ensure_milestone_escrows_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(milestone_escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("id") {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS milestone_escrows (
                id TEXT PRIMARY KEY,
                lock_tx_id TEXT NOT NULL,
                buyer_address TEXT NOT NULL,
                seller_address TEXT NOT NULL,
                total_amount INTEGER NOT NULL,
                milestone_amounts TEXT NOT NULL,
                milestone_timeouts TEXT NOT NULL,
                current_milestone INTEGER DEFAULT 0,
                milestone_statuses TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'active',
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_milestone_escrows_buyer ON milestone_escrows(buyer_address)"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_milestone_escrows_seller ON milestone_escrows(seller_address)"
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn ensure_escrow_lifecycle_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("disputed_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN disputed_at INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("dispute_reason") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_reason TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("cancelled_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN cancelled_at INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("expired_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN expired_at INTEGER")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_price_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_at_creation") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_at_creation REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_currency") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_currency TEXT DEFAULT 'USD'")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_offers_price_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(offers)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_type TEXT DEFAULT 'fixed'")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_offset") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_offset REAL DEFAULT 0.0")
            .execute(pool)
            .await?;
    }
    if !existing.contains("min_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN min_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("max_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN max_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("current_price") {
        sqlx::query("ALTER TABLE offers ADD COLUMN current_price REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_currency") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_currency TEXT DEFAULT 'USD'")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_updated_at") {
        sqlx::query("ALTER TABLE offers ADD COLUMN price_updated_at INTEGER")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_trade_hash(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("trade_hash") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN trade_hash TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_market_order_fields(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_lock_time") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_lock_time INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_at_settlement") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_at_settlement REAL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("price_source") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_source TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_escrow_price_type(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_type TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_price_type_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("price_type") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN price_type TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_dispute_mode_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("dispute_mode") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_mode TEXT DEFAULT 'standard'")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_mediator_key_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("mediator_key") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediator_key TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_lock_tx_id_index(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Add unique index on lock_tx_id + lock_tx_output_index to prevent
    // duplicate escrows for the same UTXO
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_escrows_lock_tx
         ON escrows(lock_tx_id, lock_tx_output_index)",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn ensure_vault_sweep_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(vaults)")
        .fetch_all(pool)
        .await?;
    let existing: std::collections::HashSet<String> = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect();
    if !existing.contains("owner_pubkey_hex") {
        sqlx::query("ALTER TABLE vaults ADD COLUMN owner_pubkey_hex TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("sweep_tx_id") {
        sqlx::query("ALTER TABLE vaults ADD COLUMN sweep_tx_id TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_dispute_outcome_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("dispute_outcome") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_outcome TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("dispute_resolved_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN dispute_resolved_at INTEGER")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_escrow_invoice_id_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("invoice_id") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN invoice_id TEXT REFERENCES invoices(id)")
            .execute(pool)
            .await?;
    }
    Ok(())
}

/// Add memo/notes column to escrows.
pub async fn ensure_escrow_memo_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("memo") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN memo TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_auto_settle_column(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("auto_settle_timeout") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN auto_settle_timeout INTEGER")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_deposits_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(deposits)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("id") {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS deposits (
                id TEXT PRIMARY KEY,
                escrow_id TEXT NOT NULL,
                party1_address TEXT NOT NULL,
                party2_address TEXT NOT NULL,
                deposit_amount INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'locked',
                deposit_tx_id TEXT,
                timeout INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                released_at INTEGER,
                forfeited_at INTEGER,
                forfeited_to TEXT
            )"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_deposits_escrow_id ON deposits(escrow_id)"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_deposits_status ON deposits(status)"
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn ensure_dispute_escalation_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(jury_cases)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("escalation_level") {
        sqlx::query("ALTER TABLE jury_cases ADD COLUMN escalation_level INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
    }
    if !existing.contains("escalation_deadline") {
        sqlx::query("ALTER TABLE jury_cases ADD COLUMN escalation_deadline INTEGER")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_log") {
        sqlx::query("ALTER TABLE jury_cases ADD COLUMN mediation_log TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}

pub async fn ensure_chat_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();
    if !existing.contains("chat_pubkey_buyer") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN chat_pubkey_buyer TEXT")
            .execute(pool)
            .await?;
    }
    if !existing.contains("chat_pubkey_seller") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN chat_pubkey_seller TEXT")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn ensure_mediation_columns(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    let rows = sqlx::query("PRAGMA table_info(escrows)")
        .fetch_all(pool)
        .await?;
    let existing = rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .collect::<HashSet<_>>();

    if !existing.contains("mediation_status") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_status TEXT DEFAULT NULL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_buyer_claim") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_buyer_claim TEXT DEFAULT NULL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_seller_claim") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_seller_claim TEXT DEFAULT NULL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_result") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_result TEXT DEFAULT NULL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_expires_at") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_expires_at INTEGER DEFAULT NULL")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_buyer_accepted") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_buyer_accepted INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
    }
    if !existing.contains("mediation_seller_accepted") {
        sqlx::query("ALTER TABLE escrows ADD COLUMN mediation_seller_accepted INTEGER DEFAULT 0")
            .execute(pool)
            .await?;
    }

    Ok(())
}
