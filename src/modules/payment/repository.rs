use async_trait::async_trait;
use uuid::Uuid;
use super::domain::Payment;
use super::errors::PaymentError;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, payment: &Payment) -> Result<(), PaymentError>;
    #[allow(dead_code)]
    async fn find_by_id(&self, id: Uuid) -> Result<Payment, PaymentError>;
    async fn find_by_signature(&self, signature: &str) -> Result<Option<Payment>, PaymentError>;
    async fn find_by_invoice_id(&self, invoice_id: Uuid) -> Result<Vec<Payment>, PaymentError>;
}
