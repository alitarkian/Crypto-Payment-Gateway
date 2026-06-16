use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use super::handlers::{create_wallet, generate_wallet, get_wallet, list_merchant_wallets, WalletState};

pub fn wallet_routes(state: Arc<WalletState>) -> Router {
    Router::new()
        .route("/api/v1/wallets", post(create_wallet))
        .route("/api/v1/wallets/generate", post(generate_wallet))
        .route("/api/v1/wallets/{id}", get(get_wallet))
        .route("/api/v1/merchants/{merchant_id}/wallets", get(list_merchant_wallets))
        .with_state(state)
}