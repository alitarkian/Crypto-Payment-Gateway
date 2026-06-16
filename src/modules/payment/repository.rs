use async_trait::async_trait;
use uuid::Uuid;
use super::domain::Payment;
use super::errors::PaymentError;

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn save(&self, payment: &Payment) -> Result<(), PaymentError>;
    async fn update(&self, payment: &Payment) -> Result<(), PaymentError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Payment, PaymentError>;
    async fn find_by_signature(&self, signature: &str) -> Result<Option<Payment>, PaymentError>;
    async fn find_by_invoice_id(&self, invoice_id: Uuid) -> Result<Vec<Payment>, PaymentError>;

    /// Returns all known transaction signatures for a given blockchain.
    /// Used by MultiChainWatcher to pre-seed seen_signatures on startup,
    /// preventing redundant RPC calls after a service restart.
    async fn find_signatures_by_blockchain(
        &self,
        blockchain: &str,
    ) -> Result<Vec<String>, PaymentError>;
}
