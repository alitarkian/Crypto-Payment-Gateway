use std::sync::Arc;
use uuid::Uuid;
use tracing::info;

use super::domain::Wallet;
use super::errors::WalletError;
use super::repository::WalletRepository;

pub struct CreateWallet {
    pub merchant_id: Uuid,
    pub address: String,
    pub blockchain: super::domain::Blockchain,
    pub asset: super::domain::Asset,
}

pub struct WalletUseCase {
    repo: Arc<dyn WalletRepository>,
}

impl WalletUseCase {
    pub fn new(repo: Arc<dyn WalletRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, cmd: CreateWallet) -> Result<Wallet, WalletError> {
        if let Some(_) = self.repo.find_by_address(&cmd.address).await? {
            return Err(WalletError::AddressAlreadyExists);
        }

        let wallet = Wallet::new(cmd.merchant_id, cmd.address, cmd.blockchain, cmd.asset)?;

        self.repo.save(&wallet).await?;

        info!(wallet_id = %wallet.id, merchant_id = %wallet.merchant_id, "Wallet created");

        Ok(wallet)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Wallet, WalletError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Wallet>, WalletError> {
        self.repo.find_by_merchant_id(merchant_id).await
    }
}