use async_trait::async_trait;
use uuid::Uuid;

use super::domain::{ Settlement, SettlementBatch };
use super::errors::SettlementError;

#[async_trait]
pub trait SettlementRepository: Send + Sync {
    async fn save_settlement(&self, settlement: &Settlement) -> Result<(), SettlementError>;
    async fn update_settlement(&self, settlement: &Settlement) -> Result<(), SettlementError>;
    #[allow(dead_code)]
    async fn find_settlement_by_id(&self, id: Uuid) -> Result<Settlement, SettlementError>;
    async fn find_settlement_by_payment_id(
        &self,
        payment_id: Uuid
    ) -> Result<Option<Settlement>, SettlementError>;
    async fn find_pending_settlements(&self) -> Result<Vec<Settlement>, SettlementError>;
    async fn find_settlements_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Settlement>, SettlementError>;

    async fn save_batch(&self, batch: &SettlementBatch) -> Result<(), SettlementError>;
    async fn update_batch(&self, batch: &SettlementBatch) -> Result<(), SettlementError>;
    #[allow(dead_code)]
    async fn find_batch_by_id(&self, id: Uuid) -> Result<SettlementBatch, SettlementError>;
    async fn find_open_batch_for_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Option<SettlementBatch>, SettlementError>;
    #[allow(dead_code)]
    async fn find_batches_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<SettlementBatch>, SettlementError>;
}
