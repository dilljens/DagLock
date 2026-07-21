//! Invoice API handlers — escrow-based invoicing for freelancers.
//!
//! Invoices are lightweight metadata wrappers around the standard escrow
//! covenant. Creating an invoice generates a shareable link. When the
//! client pays, a standard escrow is created with the invoice_id linked.
//! No covenant changes needed.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::api::AppState;
use crate::auth::AuthContext;
use crate::db::queries;
use crate::types::*;

#[derive(Deserialize)]
pub struct InvoiceQuery {
    pub address: Option<String>,
}

/// POST /v1/invoices — create a new invoice.
///
/// Requires auth headers proving ownership of the freelancer address.
/// Returns the invoice ID and shareable link.
pub async fn create(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    // Auth
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new("unauthorized", e.to_string()))),
        )
    })?;

    // Verify signature
    let expected_message = format!("create:invoice:{}", auth.address);
    if !state
        .sig_verifier
        .verify_signature(&auth.address, &auth.signature, &expected_message)
        .map_err(|e| {
            (
                StatusCode::FORBIDDEN,
                Json(json!(ApiError::new(
                    "forbidden",
                    format!("Signature verification failed: {e}")
                ))),
            )
        })?
    {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ApiError::new("forbidden", "Invalid signature"))),
        ));
    }

    // Validate
    if body.description.is_empty() || body.description.len() > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_description",
                "Description must be 1-500 characters"
            ))),
        ));
    }
    if body.amount_sompi <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!(ApiError::new(
                "invalid_amount",
                "Amount must be positive"
            ))),
        ));
    }

    let now = chrono::Utc::now().timestamp();
    let invoice_id = format!("INV_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));

    let invoice = Invoice {
        id: invoice_id.clone(),
        freelancer_address: auth.address.clone(),
        client_address: None,
        escrow_id: None,
        description: body.description,
        amount_sompi: body.amount_sompi,
        due_date: body.due_date,
        status: "draft".to_string(),
        created_at: now,
        paid_at: None,
        settled_at: None,
    };

    queries::insert_invoice(&state.db, &invoice)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new(
                    "database_error",
                    format!("Failed to create invoice: {e}")
                ))),
            )
        })?;

    let link = format!("https://daglock.com/pay/{}", invoice_id);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": invoice_id,
            "link": link,
            "invoice": invoice,
        })),
    ))
}

/// GET /v1/invoices/:id — public invoice details.
///
/// No auth required. Returns invoice metadata + linked escrow status
/// if paid.
pub async fn get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let invoice = queries::get_invoice(&state.db, &id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("database_error", format!("{e}")))),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!(ApiError::new(
                    "invoice_not_found",
                    format!("Invoice '{id}' not found")
                ))),
            )
        })?;

    // If linked to an escrow, fetch the escrow status too
    let escrow_status: Option<String> = if let Some(ref escrow_id) = invoice.escrow_id {
        queries::get_escrow(&state.db, escrow_id)
            .await
            .ok()
            .flatten()
            .map(|e| format!("{:?}", e.status))
    } else {
        None
    };

    Ok(Json(json!({
        "invoice": invoice,
        "escrow_status": escrow_status,
        "link": format!("https://daglock.com/pay/{}", id),
    })))
}

/// GET /v1/invoices?address=... — list invoices for an address.
pub async fn list(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(params): Query<InvoiceQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let auth = AuthContext::from_headers(&headers).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!(ApiError::new("unauthorized", e.to_string()))),
        )
    })?;

    let address = params.address.unwrap_or(auth.address.clone());

    // Auth: only allow listing your own invoices
    if auth.address != address {
        // Verify they have access to this address
        let expected_message = format!("list:invoices:{}", address);
        if !state
            .sig_verifier
            .verify_signature(&auth.address, &auth.signature, &expected_message)
            .unwrap_or(false)
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!(ApiError::new(
                    "forbidden",
                    "You can only list your own invoices"
                ))),
            ));
        }
    }

    let invoices = queries::list_invoices_by_freelancer(&state.db, &address)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!(ApiError::new("database_error", format!("{e}")))),
            )
        })?;

    Ok(Json(
        json!({ "invoices": invoices, "total": invoices.len() }),
    ))
}
