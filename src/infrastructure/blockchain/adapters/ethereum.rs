use async_trait::async_trait;
use std::collections::HashSet;
use tracing::warn;

use crate::infrastructure::blockchain::{
    chain_adapter::{ChainAdapter, DetectedPayment},
    ethereum_client::EthereumRpcClient,
};

pub struct EthereumAdapter {
    rpc: EthereumRpcClient,
    usdc_contract: String,
}

impl EthereumAdapter {
    pub fn new(rpc: EthereumRpcClient, usdc_contract: String) -> Self {
        Self { rpc, usdc_contract }
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    fn chain_name(&self) -> &str {
        "ethereum"
    }

    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen: &mut HashSet<String>,
    ) -> anyhow::Result<Vec<DetectedPayment>> {
        let mut payments = Vec::new();

        // آخر 100 block رو scan می‌کنیم
        let latest = match self.rpc.get_block_number().await {
            Ok(n) => n,
            Err(e) => {
                warn!(error = %e, "Ethereum: failed to get block number");
                return Ok(payments);
            }
        };

        let from_block = format!("0x{:x}", latest.saturating_sub(100));

        let logs = match self
            .rpc
            .get_usdc_transfers_to(wallet_address, &self.usdc_contract, &from_block)
            .await
        {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, wallet = %wallet_address, "Ethereum: failed to fetch logs");
                return Ok(payments);
            }
        };

        for log in logs {
            if log.removed.unwrap_or(false) {
                continue;
            }
            if seen.contains(&log.transaction_hash) {
                continue;
            }
            seen.insert(log.transaction_hash.clone());

            if let Some(amount) = EthereumRpcClient::decode_transfer_amount(&log.data) {
                if amount > rust_decimal::Decimal::ZERO {
                    payments.push(DetectedPayment {
                        signature: log.transaction_hash,
                        wallet_address: wallet_address.to_string(),
                        amount,
                        blockchain: "ethereum".to_string(),
                    });
                }
            }
        }

        Ok(payments)
    }
}