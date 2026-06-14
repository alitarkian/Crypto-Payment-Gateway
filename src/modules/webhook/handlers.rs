use axum::{ extract::{ Path, State }, http::StatusCode, Json };
use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use uuid::Uuid;

use super::{ errors::WebhookError, use_cases::{ RegisterWebhook, WebhookUseCase } };

pub struct WebhookState {
    pub use_case: Arc<WebhookUseCase>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterWebhookRequest {
    pub merchant_id: Uuid,
    pub url: String,
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookResponse {
    pub id: String,
    pub merchant_id: String,
    pub url: String,
    pub is_active: bool,
    pub created_at: String,
}

pub async fn register_webhook(
    State(state): State<Arc<WebhookState>>,
    Json(body): Json<RegisterWebhookRequest>
) -> Result<(StatusCode, Json<WebhookResponse>), AppError> {
    let webhook = state.use_case.register(RegisterWebhook {
        merchant_id: body.merchant_id,
        url: body.url,
        secret: body.secret,
    }).await?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookResponse {
            id: webhook.id.to_string(),
            merchant_id: webhook.merchant_id.to_string(),
            url: webhook.url,
            is_active: webhook.is_active,
            created_at: webhook.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn list_merchant_webhooks(
    State(state): State<Arc<WebhookState>>,
    Path(merchant_id): Path<Uuid>
) -> Result<Json<serde_json::Value>, AppError> {
    let webhooks = state.use_case.list_by_merchant(merchant_id).await?;

    Ok(
        Json(
            serde_json::json!({ "webhooks": webhooks.into_iter().map(|w| serde_json::json!({
        "id": w.id,
        "merchant_id": w.merchant_id,
        "url": w.url,
        "is_active": w.is_active,
        "created_at": w.created_at,
    })).collect::<Vec<_>>() })
        )
    )
}

pub struct AppError(WebhookError);

impl From<WebhookError> for AppError {
    fn from(e: WebhookError) -> Self {
        AppError(e)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            WebhookError::NotFound => (StatusCode::NOT_FOUND, "Webhook not found"),
            WebhookError::InvalidUrl(_) => (StatusCode::BAD_REQUEST, "Invalid URL"),
            WebhookError::MerchantNotFound => (StatusCode::NOT_FOUND, "Merchant not found"),
            WebhookError::EventNotFound => (StatusCode::NOT_FOUND, "Event not found"),
            WebhookError::DeliveryFailed(_) => (StatusCode::BAD_GATEWAY, "Delivery failed"),
            WebhookError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
