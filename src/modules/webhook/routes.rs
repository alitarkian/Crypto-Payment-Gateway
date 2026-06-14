use axum::{ routing::{ get, post }, Router };
use std::sync::Arc;
use super::handlers::{ register_webhook, list_merchant_webhooks, WebhookState };

pub fn webhook_routes(state: Arc<WebhookState>) -> Router {
    Router::new()
        .route("/api/v1/webhooks", post(register_webhook))
        .route("/api/v1/merchants/{merchant_id}/webhooks", get(list_merchant_webhooks))
        .with_state(state)
}
