use async_trait::async_trait;
use std::collections::HashSet;
use tracing::warn;

use crate::infrastructure::blockchain::{
    chain_adapter::{ ChainAdapter, DetectedPayment },
    rpc_client::SolanaRpcClient,
};

pub struct SolanaAdapter {
    rpc: SolanaRpcClient,
    usdc_mint: String,
}

impl SolanaAdapter {
    pub fn new(rpc: SolanaRpcClient, usdc_mint: String) -> Self {
        Self { rpc, usdc_mint }
    }

    fn extract_usdc_received(
        &self,
        pre: &Option<Vec<crate::infrastructure::blockchain::rpc_client::TokenBalance>>,
        post: &Option<Vec<crate::infrastructure::blockchain::rpc_client::TokenBalance>>
    ) -> Option<rust_decimal::Decimal> {
        use rust_decimal::Decimal;
        use std::str::FromStr;

        let post_balances = post.as_ref()?;

        for balance in post_balances {
            if balance.mint != self.usdc_mint {
                continue;
            }

            let post_amount = Decimal::from_str(&balance.ui_token_amount.ui_amount_string).ok()?;

            let pre_amount = pre
                .as_ref()
                .and_then(|pre_balances| {
                    pre_balances
                        .iter()
                        .find(|b| {
                            b.mint == self.usdc_mint && b.account_index == balance.account_index
                        })
                        .and_then(|b| {
                            Decimal::from_str(&b.ui_token_amount.ui_amount_string).ok()
                        })
                })
                .unwrap_or(Decimal::ZERO);

            let diff = post_amount - pre_amount;
            if diff > Decimal::ZERO {
                return Some(diff);
            }
        }

        None
    }
}

#[async_trait]
impl ChainAdapter for SolanaAdapter {
    fn chain_name(&self) -> &str {
        "solana"
    }

    async fn detect_payments(
        &self,
        wallet_address: &str,
        seen: &mut HashSet<String>
    ) -> anyhow::Result<Vec<DetectedPayment>> {
        let mut payments = Vec::new();

        let signatures = match self.rpc.get_signatures_for_address(wallet_address, 10).await {
            Ok(s) => s,
            Err(e) => {
                warn!(error = %e, wallet = %wallet_address, "Solana: failed to fetch signatures");
                return Ok(payments);
            }
        };

        for sig in signatures {
            if seen.contains(&sig.signature) || sig.err.is_some() {
                continue;
            }
            seen.insert(sig.signature.clone());

            let tx = match self.rpc.get_transaction(&sig.signature).await {
                Ok(Some(tx)) => tx,
                _ => {
                    continue;
                }
            };

            if let Some(meta) = &tx.meta {
                if meta.err.is_some() {
                    continue;
                }
                if
                    let Some(amount) = self.extract_usdc_received(
                        &meta.pre_token_balances,
                        &meta.post_token_balances
                    )
                {
                    payments.push(DetectedPayment {
                        signature: sig.signature.clone(),
                        wallet_address: wallet_address.to_string(),
                        amount,
                        blockchain: "solana".to_string(),
                        asset: "USDC".to_string(),
                    });
                }
            }
        }

        Ok(payments)
    }
}
