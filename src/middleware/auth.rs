use axum::{
    extract::{ Request, State },
    http::StatusCode,
    middleware::Next,
    response::{ IntoResponse, Response },
    Json,
};
use std::sync::Arc;
use crate::modules::merchant::repository::MerchantRepository;

#[derive(Clone)]
pub struct AuthState {
    pub merchant_repo: Arc<dyn MerchantRepository>,
}

pub async fn api_key_auth(
    State(state): State<AuthState>,
    mut req: Request,
    next: Next
) -> Response {
    let api_key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let Some(key) = api_key else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Missing x-api-key header" })),
        ).into_response();
    };

    match state.merchant_repo.find_by_api_key(&key).await {
        Ok(Some(merchant)) if merchant.is_active => {
            req.extensions_mut().insert(merchant);
            next.run(req).await
        }
        Ok(Some(_)) =>
            (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({ "error": "Merchant account is inactive" })),
            ).into_response(),
        Ok(None) =>
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Invalid API key" })),
            ).into_response(),
        Err(_) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Authentication error" })),
            ).into_response(),
    }
}
