use axum::{extract::Path, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::{
    errors::MerchantError,
    use_cases::{CreateMerchant, MerchantUseCase},
};

pub struct MerchantState {
    pub use_case: MerchantUseCase,
}

#[derive(Debug, Deserialize)]
pub struct CreateMerchantRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct MerchantResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: String,
}

pub async fn create_merchant(
    State(state): State<Arc<MerchantState>>,
    Json(body): Json<CreateMerchantRequest>,
) -> Result<(StatusCode, Json<MerchantResponse>), AppError> {
    let merchant = state
        .use_case
        .create(CreateMerchant {
            name: body.name,
            email: body.email,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(MerchantResponse {
            id: merchant.id.to_string(),
            name: merchant.name,
            email: merchant.email,
            api_key: merchant.api_key,
            is_active: merchant.is_active,
            created_at: merchant.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn get_merchant(
    State(state): State<Arc<MerchantState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<MerchantResponse>, AppError> {
    let merchant = state.use_case.get_by_id(id).await?;

    Ok(Json(MerchantResponse {
        id: merchant.id.to_string(),
        name: merchant.name,
        email: merchant.email,
        api_key: merchant.api_key,
        is_active: merchant.is_active,
        created_at: merchant.created_at.to_rfc3339(),
    }))
}

pub struct AppError(MerchantError);

impl From<MerchantError> for AppError {
    fn from(e: MerchantError) -> Self {
        AppError(e)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            MerchantError::NotFound => (StatusCode::NOT_FOUND, "Merchant not found"),
            MerchantError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            MerchantError::InvalidEmail => (StatusCode::BAD_REQUEST, "Invalid email address"),
            MerchantError::EmptyName => (StatusCode::BAD_REQUEST, "Name cannot be empty"),
            MerchantError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}