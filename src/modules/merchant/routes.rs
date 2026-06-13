use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use super::handlers::{create_merchant, get_merchant, MerchantState};

pub fn merchant_routes(state: Arc<MerchantState>) -> Router {
    Router::new()
        .route("/api/v1/merchants", post(create_merchant))
        .route("/api/v1/merchants/{id}", get(get_merchant))
        .with_state(state)
}