use sqlx::{Pool, Row, Sqlite};

use crate::types::Invoice;

/// Insert a new invoice.
pub async fn insert_invoice(pool: &Pool<Sqlite>, invoice: &Invoice) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO invoices (id, freelancer_address, client_address, escrow_id, description, amount_sompi, due_date, status, created_at, paid_at, settled_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )
    .bind(&invoice.id)
    .bind(&invoice.freelancer_address)
    .bind(&invoice.client_address)
    .bind(&invoice.escrow_id)
    .bind(&invoice.description)
    .bind(invoice.amount_sompi)
    .bind(invoice.due_date)
    .bind(&invoice.status)
    .bind(invoice.created_at)
    .bind(invoice.paid_at)
    .bind(invoice.settled_at)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get an invoice by ID (public).
pub async fn get_invoice(pool: &Pool<Sqlite>, id: &str) -> Result<Option<Invoice>, sqlx::Error> {
    let row = sqlx::query("SELECT id, freelancer_address, client_address, escrow_id, description, amount_sompi, due_date, status, created_at, paid_at, settled_at FROM invoices WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    match row {
        Some(r) => Ok(Some(Invoice {
            id: r.try_get("id")?,
            freelancer_address: r.try_get("freelancer_address")?,
            client_address: r.try_get("client_address")?,
            escrow_id: r.try_get("escrow_id")?,
            description: r.try_get("description")?,
            amount_sompi: r.try_get("amount_sompi")?,
            due_date: r.try_get("due_date")?,
            status: r.try_get("status")?,
            created_at: r.try_get("created_at")?,
            paid_at: r.try_get("paid_at")?,
            settled_at: r.try_get("settled_at")?,
        })),
        None => Ok(None),
    }
}

/// List invoices for a freelancer address.
pub async fn list_invoices_by_freelancer(
    pool: &Pool<Sqlite>,
    freelancer_address: &str,
) -> Result<Vec<Invoice>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT id, freelancer_address, client_address, escrow_id, description, amount_sompi, due_date, status, created_at, paid_at, settled_at
         FROM invoices WHERE freelancer_address = ?1 ORDER BY created_at DESC",
    )
    .bind(freelancer_address)
    .fetch_all(pool)
    .await?;

    let invoices = rows
        .into_iter()
        .map(|r| Invoice {
            id: r.try_get("id").unwrap_or_default(),
            freelancer_address: r.try_get("freelancer_address").unwrap_or_default(),
            client_address: r.try_get("client_address").unwrap_or(None),
            escrow_id: r.try_get("escrow_id").unwrap_or(None),
            description: r.try_get("description").unwrap_or_default(),
            amount_sompi: r.try_get("amount_sompi").unwrap_or(0),
            due_date: r.try_get("due_date").unwrap_or(None),
            status: r.try_get("status").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap_or(0),
            paid_at: r.try_get("paid_at").unwrap_or(None),
            settled_at: r.try_get("settled_at").unwrap_or(None),
        })
        .collect();

    Ok(invoices)
}

/// Update invoice status.
pub async fn update_invoice_status(
    pool: &Pool<Sqlite>,
    id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE invoices SET status = ?1 WHERE id = ?2")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Link an invoice to an escrow (when paid).
pub async fn link_invoice_to_escrow(
    pool: &Pool<Sqlite>,
    invoice_id: &str,
    escrow_id: &str,
    client_address: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE invoices SET status = 'paid', escrow_id = ?1, client_address = ?2, paid_at = ?3 WHERE id = ?4",
    )
    .bind(escrow_id)
    .bind(client_address)
    .bind(chrono::Utc::now().timestamp())
    .bind(invoice_id)
    .execute(pool)
    .await?;
    Ok(())
}
