use async_trait::async_trait;
use uuid::Uuid;
use super::domain::Wallet;
use super::errors::WalletError;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn save(&self, wallet: &Wallet) -> Result<(), WalletError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Wallet, WalletError>;
    async fn find_by_merchant_id(&self, merchant_id: Uuid) -> Result<Vec<Wallet>, WalletError>;
    async fn find_by_address(&self, address: &str) -> Result<Option<Wallet>, WalletError>;
}