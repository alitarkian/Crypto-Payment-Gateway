use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct DetectedPayment {
    pub signature: String,
    pub wallet_address: String,
    pub amount: Decimal,
    pub blockchain: String,
    /// The asset symbol detected (e.g. "USDC")
    pub asset: String,
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
