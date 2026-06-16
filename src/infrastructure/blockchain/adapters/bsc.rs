use async_trait::async_trait;
use std::collections::HashSet;
use tracing::warn;

use crate::infrastructure::blockchain::{
    chain_adapter::{ChainAdapter, DetectedPayment},
    ethereum_client::EthereumRpcClient,
};

/// BNB Smart Chain adapter.
///
/// BSC is EVM-compatible — we reuse EthereumRpcClient with a BSC RPC endpoint.
/// Detects BEP-20 USDT (and optionally USDC) transfers to watched addresses.
pub struct BscAdapter {
    rpc: EthereumRpcClient,
    /// BEP-20 token contract address to monitor (e.g. USDT on BSC)
    token_contract: String,
    /// Asset symbol this contract represents (e.g. "USDT")
    asset_symbol: String,
}

impl BscAdapter {
    pub fn new(rpc: EthereumRpcClient, token_contract: String, asset_symbol: String) -> Self {
        Self { rpc, token_contract, asset_symbol }
    }
}

#[async_trait]
impl ChainAdapter for BscAdapter {
    fn chain_name(&self) -> &str {
        "bsc"
    }

    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen: &mut HashSet<String>,
    ) -> anyhow::Result<Vec<DetectedPayment>> {
        let mut payments = Vec::new();

        let latest = match self.rpc.get_block_number().await {
            Ok(n) => n,
            Err(e) => {
                warn!(error = %e, "BSC: failed to get block number");
                return Ok(payments);
            }
        };

        // BSC produces ~3s blocks — scan last 200 blocks (~10 minutes)
        let from_block = format!("0x{:x}", latest.saturating_sub(200));

        let logs = match self
            .rpc
            .get_usdc_transfers_to(wallet_address, &self.token_contract, &from_block)
            .await
        {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, wallet = %wallet_address, "BSC: failed to fetch token logs");
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
                        blockchain: "bsc".to_string(),
                        asset: self.asset_symbol.clone(),
                    });
                }
            }
        }

        Ok(payments)
    }
}
