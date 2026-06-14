use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct DetectedPayment {
    pub signature: String,
    #[allow(dead_code)]
    pub wallet_address: String,
    pub amount: Decimal,
    #[allow(dead_code)]
    pub blockchain: String,
}

#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_name(&self) -> &str;

    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen_signatures: &mut std::collections::HashSet<String>,
    ) -> anyhow::Result<Vec<DetectedPayment>>;
}
