use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::{
    errors::InvoiceError,
    use_cases::{CreateInvoice, InvoiceUseCase},
};

pub struct InvoiceState {
    pub use_case: InvoiceUseCase,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvoiceRequest {
    pub merchant_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: Decimal,
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: String,
    pub merchant_id: String,
    pub wallet_id: String,
    pub amount: String,
    pub asset: String,
    pub blockchain: String,
    pub status: String,
    pub description: Option<String>,
    pub expires_at: String,
    pub paid_at: Option<String>,
    pub created_at: String,
}

pub async fn create_invoice(
    State(state): State<Arc<InvoiceState>>,
    Json(body): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), AppError> {
    let invoice = state
        .use_case
        .create(CreateInvoice {
            merchant_id: body.merchant_id,
            wallet_id: body.wallet_id,
            amount: body.amount,
            description: body.description,
            expires_at: body.expires_at,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(InvoiceResponse {
            id: invoice.id.to_string(),
            merchant_id: invoice.merchant_id.to_string(),
            wallet_id: invoice.wallet_id.to_string(),
            amount: invoice.amount.to_string(),
            asset: invoice.asset,
            blockchain: invoice.blockchain,
            status: invoice.status.as_str().to_string(),
            description: invoice.description,
            expires_at: invoice.expires_at.to_rfc3339(),
            paid_at: invoice.paid_at.map(|t| t.to_rfc3339()),
            created_at: invoice.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn get_invoice(
    State(state): State<Arc<InvoiceState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let invoice = state.use_case.get_by_id(id).await?;

    Ok(Json(InvoiceResponse {
        id: invoice.id.to_string(),
        merchant_id: invoice.merchant_id.to_string(),
        wallet_id: invoice.wallet_id.to_string(),
        amount: invoice.amount.to_string(),
        asset: invoice.asset,
        blockchain: invoice.blockchain,
        status: invoice.status.as_str().to_string(),
        description: invoice.description,
        expires_at: invoice.expires_at.to_rfc3339(),
        paid_at: invoice.paid_at.map(|t| t.to_rfc3339()),
        created_at: invoice.created_at.to_rfc3339(),
    }))
}

pub async fn list_merchant_invoices(
    State(state): State<Arc<InvoiceState>>,
    Path(merchant_id): Path<Uuid>,
) -> Result<Json<Vec<InvoiceResponse>>, AppError> {
    let invoices = state.use_case.list_by_merchant(merchant_id).await?;

    Ok(Json(
        invoices
            .into_iter()
            .map(|i| InvoiceResponse {
                id: i.id.to_string(),
                merchant_id: i.merchant_id.to_string(),
                wallet_id: i.wallet_id.to_string(),
                amount: i.amount.to_string(),
                asset: i.asset,
                blockchain: i.blockchain,
                status: i.status.as_str().to_string(),
                description: i.description,
                expires_at: i.expires_at.to_rfc3339(),
                paid_at: i.paid_at.map(|t| t.to_rfc3339()),
                created_at: i.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

pub struct AppError(InvoiceError);

impl From<InvoiceError> for AppError {
    fn from(e: InvoiceError) -> Self {
        AppError(e)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            InvoiceError::NotFound => (StatusCode::NOT_FOUND, "Invoice not found"),
            InvoiceError::AlreadyPaid => (StatusCode::CONFLICT, "Invoice already paid"),
            InvoiceError::AlreadyExpired => (StatusCode::CONFLICT, "Invoice already expired"),
            InvoiceError::AlreadyCancelled => (StatusCode::CONFLICT, "Invoice already cancelled"),
            InvoiceError::InvalidAmount => (StatusCode::BAD_REQUEST, "Invalid amount"),
            InvoiceError::InvalidExpiration => (StatusCode::BAD_REQUEST, "Invalid expiration date"),
            InvoiceError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}