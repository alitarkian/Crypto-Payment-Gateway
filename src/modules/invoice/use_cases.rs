use std::sync::Arc;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use tracing::info;

use super::domain::Invoice;
use super::errors::InvoiceError;
use super::repository::InvoiceRepository;

pub struct CreateInvoice {
    pub merchant_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: Decimal,
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
}

pub struct InvoiceUseCase {
    repo: Arc<dyn InvoiceRepository>,
}

impl InvoiceUseCase {
    pub fn new(repo: Arc<dyn InvoiceRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, cmd: CreateInvoice) -> Result<Invoice, InvoiceError> {
        let invoice = Invoice::new(
            cmd.merchant_id,
            cmd.wallet_id,
            cmd.amount,
            cmd.description,
            cmd.expires_at,
        )?;

        self.repo.save(&invoice).await?;

        info!(invoice_id = %invoice.id, merchant_id = %invoice.merchant_id, amount = %invoice.amount, "Invoice created");

        Ok(invoice)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Invoice, InvoiceError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Invoice>, InvoiceError> {
        self.repo.find_by_merchant_id(merchant_id).await
    }

    pub async fn expire_pending(&self) -> Result<usize, InvoiceError> {
        let expired = self.repo.find_pending_expired().await?;
        let count = expired.len();

        for mut invoice in expired {
            invoice.mark_expired()?;
            self.repo.update(&invoice).await?;
            info!(invoice_id = %invoice.id, "Invoice expired");
        }

        Ok(count)
    }
}