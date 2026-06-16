use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{ interval, Duration };
use tracing::{ error, info, warn };

use crate::infrastructure::blockchain::chain_adapter::ChainAdapter;
use crate::modules::invoice::repository::InvoiceRepository;
use crate::modules::payment::repository::PaymentRepository;
use crate::modules::payment::use_cases::{ PaymentUseCase, ProcessPayment };
use crate::modules::wallet::repository::WalletRepository;

pub struct MultiChainWatcher {
    adapters: Vec<Arc<dyn ChainAdapter>>,
    invoice_repo: Arc<dyn InvoiceRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
    payment_repo: Arc<dyn PaymentRepository>,
    payment_use_case: Arc<PaymentUseCase>,
}

impl MultiChainWatcher {
    pub fn new(
        adapters: Vec<Arc<dyn ChainAdapter>>,
        invoice_repo: Arc<dyn InvoiceRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
        payment_repo: Arc<dyn PaymentRepository>,
        payment_use_case: Arc<PaymentUseCase>,
    ) -> Self {
        Self {
            adapters,
            invoice_repo,
            wallet_repo,
            payment_repo,
            payment_use_case,
        }
    }

    pub async fn run(self) {
        let chain_names: Vec<&str> = self.adapters.iter().map(|a| a.chain_name()).collect();
        info!(chains = ?chain_names, "MultiChainWatcher started");

        // ── Seed seen_signatures from DB to avoid redundant RPC calls after restart ──
        let mut seen: HashMap<String, HashSet<String>> = HashMap::new();
        for adapter in &self.adapters {
            let chain = adapter.chain_name().to_string();
            match self.payment_repo.find_signatures_by_blockchain(&chain).await {
                Ok(sigs) => {
                    let count = sigs.len();
                    seen.insert(chain.clone(), sigs.into_iter().collect());
                    info!(chain = %chain, count, "Seeded seen_signatures from DB");
                }
                Err(e) => {
                    error!(chain = %chain, error = %e, "Failed to seed seen_signatures — starting empty");
                    seen.insert(chain, HashSet::new());
                }
            }
        }

        let mut tick = interval(Duration::from_secs(10));
        loop {
            tick.tick().await;
            if let Err(e) = self.watch_cycle(&mut seen).await {
                error!(error = %e, "MultiChainWatcher cycle failed");
            }
        }
    }

    async fn watch_cycle(&self, seen: &mut HashMap<String, HashSet<String>>) -> anyhow::Result<()> {
        let invoices = self.invoice_repo.find_pending_active().await?;

        for invoice in &invoices {
            let wallet = match self.wallet_repo.find_by_id(invoice.wallet_id).await {
                Ok(w) => w,
                Err(e) => {
                    warn!(error = %e, wallet_id = %invoice.wallet_id, "Failed to fetch wallet");
                    continue;
                }
            };

            for adapter in &self.adapters {
                // فقط adapter مربوط به blockchain این wallet
                if adapter.chain_name() != wallet.blockchain.as_str() {
                    continue;
                }

                let chain_seen = seen.entry(adapter.chain_name().to_string()).or_default();

                let detected = match adapter.detect_payments(&wallet.address, chain_seen).await {
                    Ok(d) => d,
                    Err(e) => {
                        warn!(
                            error = %e,
                            chain = adapter.chain_name(),
                            wallet = %wallet.address,
                            "Failed to detect payments"
                        );
                        continue;
                    }
                };

                for payment in detected {
                    info!(
                        chain = adapter.chain_name(),
                        invoice_id = %invoice.id,
                        signature = %payment.signature,
                        amount = %payment.amount,
                        "Payment detected"
                    );

                    let cmd = ProcessPayment {
                        invoice_id: invoice.id,
                        wallet_id: wallet.id,
                        merchant_id: invoice.merchant_id,
                        signature: payment.signature.clone(),
                        amount: payment.amount,
                        blockchain: payment.blockchain.clone(),
                        asset: payment.asset.clone(),
                    };

                    match self.payment_use_case.process(cmd).await {
                        Ok(p) => info!(payment_id = %p.id, "Payment processed"),
                        Err(e) => warn!(error = %e, "Failed to process payment"),
                    }
                }
            }
        }

        Ok(())
    }
}
