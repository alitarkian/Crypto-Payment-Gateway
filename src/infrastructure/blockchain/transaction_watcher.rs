use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{ interval, Duration };
use tracing::{ error, info, warn };

use crate::infrastructure::blockchain::rpc_client::SolanaRpcClient;
use crate::modules::invoice::repository::InvoiceRepository;
use crate::modules::wallet::repository::WalletRepository;

pub struct TransactionWatcher {
    rpc: SolanaRpcClient,
    invoice_repo: Arc<dyn InvoiceRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
    usdc_mint: String,
}

impl TransactionWatcher {
    pub fn new(
        rpc: SolanaRpcClient,
        invoice_repo: Arc<dyn InvoiceRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
        usdc_mint: String
    ) -> Self {
        Self { rpc, invoice_repo, wallet_repo, usdc_mint }
    }

    pub async fn run(self) {
        info!("Transaction watcher started");
        let mut tick = interval(Duration::from_secs(10));
        let mut seen: HashSet<String> = HashSet::new();

        loop {
            tick.tick().await;
            if let Err(e) = self.watch_cycle(&mut seen).await {
                error!(error = %e, "Watcher cycle failed");
            }
        }
    }

    async fn watch_cycle(&self, seen: &mut HashSet<String>) -> anyhow::Result<()> {
        let invoices = self.invoice_repo.find_pending_active().await?;

        for invoice in invoices {
            let wallet = match self.wallet_repo.find_by_id(invoice.wallet_id).await {
                Ok(w) => w,
                Err(e) => {
                    warn!(error = %e, wallet_id = %invoice.wallet_id, "Failed to fetch wallet");
                    continue;
                }
            };

            let signatures = match self.rpc.get_signatures_for_address(&wallet.address, 10).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(error = %e, wallet = %wallet.address, "Failed to fetch signatures");
                    continue;
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
                        info!(
                            invoice_id = %invoice.id,
                            signature = %sig.signature,
                            amount = %amount,
                            "Payment detected"
                        );
                    }
                }
            }
        }

        Ok(())
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
                        .find(
                            |b| b.mint == self.usdc_mint && b.account_index == balance.account_index
                        )
                        .and_then(|b| Decimal::from_str(&b.ui_token_amount.ui_amount_string).ok())
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
