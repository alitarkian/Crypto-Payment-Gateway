use async_trait::async_trait;
use uuid::Uuid;
use super::domain::Merchant;
use super::errors::MerchantError;

#[async_trait]
pub trait MerchantRepository: Send + Sync {
    async fn save(&self, merchant: &Merchant) -> Result<(), MerchantError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Merchant, MerchantError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, MerchantError>;
    #[allow(dead_code)]
    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<Merchant>, MerchantError>;
    async fn find_all(&self) -> Result<Vec<Merchant>, MerchantError>;
    async fn update(&self, merchant: &Merchant) -> Result<(), MerchantError>;
}
