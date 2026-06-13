use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::{
    errors::WalletError,
    use_cases::{CreateWallet, WalletUseCase},
};

pub struct WalletState {
    pub use_case: WalletUseCase,
}

#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub merchant_id: Uuid,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub id: String,
    pub merchant_id: String,
    pub address: String,
    pub blockchain: String,
    pub asset: String,
    pub balance: String,
    pub is_active: bool,
    pub created_at: String,
}

pub async fn create_wallet(
    State(state): State<Arc<WalletState>>,
    Json(body): Json<CreateWalletRequest>,
) -> Result<(StatusCode, Json<WalletResponse>), AppError> {
    let wallet = state
        .use_case
        .create(CreateWallet {
            merchant_id: body.merchant_id,
            address: body.address,
        })
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(WalletResponse {
            id: wallet.id.to_string(),
            merchant_id: wallet.merchant_id.to_string(),
            address: wallet.address,
            blockchain: wallet.blockchain.as_str().to_string(),
            asset: wallet.asset.as_str().to_string(),
            balance: wallet.balance.to_string(),
            is_active: wallet.is_active,
            created_at: wallet.created_at.to_rfc3339(),
        }),
    ))
}

pub async fn get_wallet(
    State(state): State<Arc<WalletState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<WalletResponse>, AppError> {
    let wallet = state.use_case.get_by_id(id).await?;

    Ok(Json(WalletResponse {
        id: wallet.id.to_string(),
        merchant_id: wallet.merchant_id.to_string(),
        address: wallet.address,
        blockchain: wallet.blockchain.as_str().to_string(),
        asset: wallet.asset.as_str().to_string(),
        balance: wallet.balance.to_string(),
        is_active: wallet.is_active,
        created_at: wallet.created_at.to_rfc3339(),
    }))
}

pub async fn list_merchant_wallets(
    State(state): State<Arc<WalletState>>,
    Path(merchant_id): Path<Uuid>,
) -> Result<Json<Vec<WalletResponse>>, AppError> {
    let wallets = state.use_case.list_by_merchant(merchant_id).await?;

    Ok(Json(
        wallets
            .into_iter()
            .map(|w| WalletResponse {
                id: w.id.to_string(),
                merchant_id: w.merchant_id.to_string(),
                address: w.address,
                blockchain: w.blockchain.as_str().to_string(),
                asset: w.asset.as_str().to_string(),
                balance: w.balance.to_string(),
                is_active: w.is_active,
                created_at: w.created_at.to_rfc3339(),
            })
            .collect(),
    ))
}

pub struct AppError(WalletError);

impl From<WalletError> for AppError {
    fn from(e: WalletError) -> Self {
        AppError(e)
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self.0 {
            WalletError::NotFound => (StatusCode::NOT_FOUND, "Wallet not found"),
            WalletError::MerchantNotFound => (StatusCode::NOT_FOUND, "Merchant not found"),
            WalletError::AddressAlreadyExists => (StatusCode::CONFLICT, "Address already exists"),
            WalletError::InvalidAddress => (StatusCode::BAD_REQUEST, "Invalid address"),
            WalletError::InactiveWallet => (StatusCode::BAD_REQUEST, "Wallet is inactive"),
            WalletError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}