use std::sync::Arc;
use rust_decimal::Decimal;
use tracing::{info, warn};
use uuid::Uuid;

use super::domain::Payment;
use super::errors::PaymentError;
use super::repository::PaymentRepository;
use crate::modules::invoice::errors::InvoiceError;
use crate::modules::invoice::repository::InvoiceRepository;

pub struct ProcessPayment {
    pub invoice_id: Uuid,
    pub wallet_id: Uuid,
    pub merchant_id: Uuid,
    pub signature: String,
    pub amount: Decimal,
}

pub struct PaymentUseCase {
    payment_repo: Arc<dyn PaymentRepository>,
    invoice_repo: Arc<dyn InvoiceRepository>,
}

impl PaymentUseCase {
    pub fn new(
        payment_repo: Arc<dyn PaymentRepository>,
        invoice_repo: Arc<dyn InvoiceRepository>,
    ) -> Self {
        Self { payment_repo, invoice_repo }
    }

    pub async fn process(&self, cmd: ProcessPayment) -> Result<Payment, PaymentError> {
        if let Some(_) = self.payment_repo.find_by_signature(&cmd.signature).await? {
            warn!(signature = %cmd.signature, "Duplicate signature detected");
            return Err(PaymentError::DuplicateSignature);
        }

        let mut invoice = self.invoice_repo
            .find_by_id(cmd.invoice_id)
            .await
            .map_err(|e| match e {
                InvoiceError::NotFound => PaymentError::InvoiceNotFound,
                _ => PaymentError::DatabaseError(e.to_string()),
            })?;

        if !invoice.is_payable() {
            return Err(PaymentError::InvoiceNotPayable);
        }

        if cmd.amount < invoice.amount {
            return Err(PaymentError::AmountMismatch {
                expected: invoice.amount.to_string(),
                received: cmd.amount.to_string(),
            });
        }

        let payment = Payment::new(
            cmd.invoice_id,
            cmd.wallet_id,
            cmd.merchant_id,
            cmd.signature.clone(),
            cmd.amount,
        );

        self.payment_repo.save(&payment).await?;

        invoice.mark_paid().map_err(|e| PaymentError::DatabaseError(e.to_string()))?;
        self.invoice_repo.update(&invoice).await
            .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;

        info!(
            payment_id = %payment.id,
            invoice_id = %cmd.invoice_id,
            signature = %cmd.signature,
            amount = %cmd.amount,
            "Payment processed successfully"
        );

        Ok(payment)
    }

    pub async fn get_by_invoice(&self, invoice_id: Uuid) -> Result<Vec<Payment>, PaymentError> {
        self.payment_repo.find_by_invoice_id(invoice_id).await
    }
}