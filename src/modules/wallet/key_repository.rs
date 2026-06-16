use async_trait::async_trait;
use uuid::Uuid;

use super::errors::WalletError;
use super::key_domain::WalletKey;

#[async_trait]
pub trait WalletKeyRepository: Send + Sync {
    async fn save(&self, key: &WalletKey) -> Result<(), WalletError>;
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<WalletKey, WalletError>;
}
