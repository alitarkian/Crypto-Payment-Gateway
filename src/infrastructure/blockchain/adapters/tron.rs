use async_trait::async_trait;
use std::collections::HashSet;
use tracing::warn;

use crate::infrastructure::blockchain::{
    chain_adapter::{ChainAdapter, DetectedPayment},
    tron_client::TronClient,
};

/// Tron adapter — detects TRC-20 token transfers (USDT, USDC) via TronGrid REST API.
pub struct TronAdapter {
    client: TronClient,
    /// TRC-20 contract address to monitor (e.g. USDT: TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t)
    contract_address: String,
    /// Asset symbol for this contract (e.g. "USDT")
    asset_symbol: String,
}

impl TronAdapter {
    pub fn new(client: TronClient, contract_address: String, asset_symbol: String) -> Self {
        Self { client, contract_address, asset_symbol }
    }

    /// Returns a timestamp 10 minutes ago in milliseconds — used to bound TronGrid queries.
    fn lookback_timestamp_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Clock before epoch")
            .as_millis() as u64;
        // Look back 10 minutes (600_000 ms)
        now_ms.saturating_sub(600_000)
    }
}

#[async_trait]
impl ChainAdapter for TronAdapter {
    fn chain_name(&self) -> &str {
        "tron"
    }

    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen: &mut HashSet<String>,
    ) -> anyhow::Result<Vec<DetectedPayment>> {
        let mut payments = Vec::new();

        let min_ts = Self::lookback_timestamp_ms();

        let transfers = match self
            .client
            .get_trc20_transfers_to(wallet_address, &self.contract_address, min_ts)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    error = %e,
                    wallet = %wallet_address,
                    "Tron: failed to fetch TRC-20 transfers"
                );
                return Ok(payments);
            }
        };

        for transfer in transfers {
            // Guard: only transfers TO our wallet (TronGrid filter handles this,
            // but we double-check to prevent any API quirks)
            if !transfer.to.eq_ignore_ascii_case(wallet_address) {
                continue;
            }

            if seen.contains(&transfer.transaction_id) {
                continue;
            }
            seen.insert(transfer.transaction_id.clone());

            let amount = match transfer.amount_decimal() {
                Some(a) if a > rust_decimal::Decimal::ZERO => a,
                _ => continue,
            };

            payments.push(DetectedPayment {
                signature: transfer.transaction_id,
                wallet_address: wallet_address.to_string(),
                amount,
                blockchain: "tron".to_string(),
                asset: self.asset_symbol.clone(),
            });
        }

        Ok(payments)
    }
}
