use std::sync::Arc;
use chrono::Utc;
use tracing::{ info, warn };
use uuid::Uuid;

use super::domain::{ Settlement, SettlementBatch, PLATFORM_FEE_RATE };
use super::errors::SettlementError;
use super::repository::SettlementRepository;

pub struct CreateSettlement {
    pub merchant_id: Uuid,
    pub invoice_id: Uuid,
    pub payment_id: Uuid,
    pub gross_amount: rust_decimal::Decimal,
}

pub struct SettlementUseCase {
    repo: Arc<dyn SettlementRepository>,
}

impl SettlementUseCase {
    pub fn new(repo: Arc<dyn SettlementRepository>) -> Self {
        Self { repo }
    }

    /// پس از تایید payment — یک settlement record می‌سازه
    pub async fn create(&self, cmd: CreateSettlement) -> Result<Settlement, SettlementError> {
        // جلوگیری از duplicate
        if self.repo.find_settlement_by_payment_id(cmd.payment_id).await?.is_some() {
            warn!(payment_id = %cmd.payment_id, "Settlement already exists");
            return Err(SettlementError::AlreadyExists);
        }

        let settlement = Settlement::new(
            cmd.merchant_id,
            cmd.invoice_id,
            cmd.payment_id,
            cmd.gross_amount,
            PLATFORM_FEE_RATE
        );

        self.repo.save_settlement(&settlement).await?;

        info!(
            settlement_id = %settlement.id,
            merchant_id = %cmd.merchant_id,
            gross = %settlement.gross_amount,
            fee = %settlement.fee_amount,
            net = %settlement.net_amount,
            "Settlement created"
        );

        Ok(settlement)
    }

    /// Settlement های pending رو به batch روزانه اضافه می‌کنه
    pub async fn process_pending(&self) -> Result<usize, SettlementError> {
        let pending = self.repo.find_pending_settlements().await?;

        if pending.is_empty() {
            return Ok(0);
        }

        let today = Utc::now().date_naive();
        let mut processed = 0;

        // group by merchant
        let mut by_merchant: std::collections::HashMap<
            Uuid,
            Vec<Settlement>
        > = std::collections::HashMap::new();

        for s in pending {
            by_merchant.entry(s.merchant_id).or_default().push(s);
        }

        for (merchant_id, settlements) in by_merchant {
            // batch موجود رو پیدا کن یا بساز
            let mut batch = match self.repo.find_open_batch_for_merchant(merchant_id).await? {
                Some(b) => b,
                None => {
                    let b = SettlementBatch::new(merchant_id, today);
                    self.repo.save_batch(&b).await?;
                    b
                }
            };

            for mut settlement in settlements {
                batch.add_settlement(&settlement);
                settlement.assign_to_batch(batch.id);
                settlement.mark_ready();
                self.repo.update_settlement(&settlement).await?;
                processed += 1;
            }

            self.repo.update_batch(&batch).await?;

            info!(
                batch_id = %batch.id,
                merchant_id = %merchant_id,
                count = %batch.settlement_count,
                total_net = %batch.total_net,
                "Batch updated"
            );
        }

        Ok(processed)
    }

    #[allow(dead_code)]
    pub async fn close_daily_batches(&self) -> Result<usize, SettlementError> {
        // در این فاز فقط open batch های merchant ها رو می‌بندیم
        // on-chain transfer در فاز 9 اضافه می‌شه
        let today = Utc::now().date_naive();
        let mut closed = 0;

        // این query در repository پیاده می‌شه
        // فعلاً placeholder برای فاز 9
        info!(date = %today, "Daily batch close triggered (ready for Phase 9 payout)");
        closed += 1;

        Ok(closed)
    }

    #[allow(dead_code)]
    pub async fn get_merchant_settlements(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Settlement>, SettlementError> {
        self.repo.find_settlements_by_merchant(merchant_id).await
    }

    #[allow(dead_code)]
    pub async fn get_merchant_batches(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<SettlementBatch>, SettlementError> {
        self.repo.find_batches_by_merchant(merchant_id).await
    }
}
