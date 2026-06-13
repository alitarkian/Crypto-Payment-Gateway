use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use super::handlers::{create_invoice, get_invoice, list_merchant_invoices, InvoiceState};

pub fn invoice_routes(state: Arc<InvoiceState>) -> Router {
    Router::new()
        .route("/api/v1/invoices", post(create_invoice))
        .route("/api/v1/invoices/{id}", get(get_invoice))
        .route("/api/v1/merchants/{merchant_id}/invoices", get(list_merchant_invoices))
        .with_state(state)
}