use async_trait::async_trait;
use uuid::Uuid;
use super::domain::Invoice;
use super::errors::InvoiceError;

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn save(&self, invoice: &Invoice) -> Result<(), InvoiceError>;
    async fn update(&self, invoice: &Invoice) -> Result<(), InvoiceError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Invoice, InvoiceError>;
    async fn find_by_merchant_id(&self, merchant_id: Uuid) -> Result<Vec<Invoice>, InvoiceError>;
    async fn find_pending_expired(&self) -> Result<Vec<Invoice>, InvoiceError>;
}