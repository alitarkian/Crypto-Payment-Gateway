use std::sync::Arc;
use uuid::Uuid;
use tracing::info;
use super::domain::Merchant;
use super::errors::MerchantError;
use super::repository::MerchantRepository;

pub struct CreateMerchant {
    pub name: String,
    pub email: String,
}

pub struct MerchantUseCase {
    repo: Arc<dyn MerchantRepository>,
}

impl MerchantUseCase {
    pub fn new(repo: Arc<dyn MerchantRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, cmd: CreateMerchant) -> Result<Merchant, MerchantError> {
        if let Some(_) = self.repo.find_by_email(&cmd.email).await? {
            return Err(MerchantError::EmailAlreadyExists);
        }

        let merchant = Merchant::new(cmd.name, cmd.email)?;

        self.repo.save(&merchant).await?;

        info!(merchant_id = %merchant.id, "Merchant created");

        Ok(merchant)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Merchant, MerchantError> {
        self.repo.find_by_id(id).await
    }
}