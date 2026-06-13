use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

use crate::modules::merchant::errors::MerchantError;
use crate::modules::merchant::domain::Merchant;
use crate::modules::invoice::repository::InvoiceRepository;
use crate::modules::invoice::domain::Invoice;
use crate::modules::payment::repository::PaymentRepository;
use crate::modules::payment::domain::Payment;
use crate::modules::settlement::repository::SettlementRepository;
use crate::modules::settlement::domain::Settlement;
use crate::modules::settlement::use_cases::SettlementUseCase;
use crate::modules::webhook::repository::WebhookRepository;
use crate::modules::webhook::domain::WebhookEvent;
use crate::modules::webhook::use_cases::WebhookUseCase;
use crate::modules::merchant::repository::MerchantRepository;

use super::audit::{ AuditLogger, LogAction };

pub struct AdminUseCase {
    pub merchant_repo: Arc<dyn MerchantRepository>,
    pub invoice_repo: Arc<dyn InvoiceRepository>,
    pub payment_repo: Arc<dyn PaymentRepository>,
    pub settlement_repo: Arc<dyn SettlementRepository>,
    pub settlement_use_case: Arc<SettlementUseCase>,
    pub webhook_repo: Arc<dyn WebhookRepository>,
    pub webhook_use_case: Arc<WebhookUseCase>,
    pub audit: AuditLogger,
}

impl AdminUseCase {
    pub async fn list_merchants(&self) -> Result<Vec<Merchant>, MerchantError> {
        self.merchant_repo.find_all().await
    }

    pub async fn get_merchant(&self, id: Uuid) -> Result<Merchant, MerchantError> {
        self.merchant_repo.find_by_id(id).await
    }

    pub async fn activate_merchant(
        &self,
        id: Uuid,
        actor: String
    ) -> Result<Merchant, MerchantError> {
        let mut merchant = self.merchant_repo.find_by_id(id).await?;
        merchant.is_active = true;
        self.merchant_repo.update(&merchant).await?;
        self.audit.log(LogAction {
            action: "admin.merchant_activated",
            actor,
            entity_type: "merchant",
            entity_id: id,
            merchant_id: Some(id),
            metadata: None,
            ip_address: None,
        }).await;
        info!(merchant_id = %id, "Merchant activated by admin");
        Ok(merchant)
    }

    pub async fn deactivate_merchant(
        &self,
        id: Uuid,
        actor: String
    ) -> Result<Merchant, MerchantError> {
        let mut merchant = self.merchant_repo.find_by_id(id).await?;
        merchant.is_active = false;
        self.merchant_repo.update(&merchant).await?;
        self.audit.log(LogAction {
            action: "admin.merchant_deactivated",
            actor,
            entity_type: "merchant",
            entity_id: id,
            merchant_id: Some(id),
            metadata: None,
            ip_address: None,
        }).await;
        info!(merchant_id = %id, "Merchant deactivated by admin");
        Ok(merchant)
    }

    pub async fn list_invoices_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Invoice>, crate::modules::invoice::errors::InvoiceError> {
        self.invoice_repo.find_by_merchant_id(merchant_id).await
    }

    pub async fn list_payments_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Payment>, crate::modules::payment::errors::PaymentError> {
        self.payment_repo.find_by_invoice_id(merchant_id).await
    }

    pub async fn list_settlements_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Settlement>, crate::modules::settlement::errors::SettlementError> {
        self.settlement_repo.find_settlements_by_merchant(merchant_id).await
    }

    pub async fn trigger_settlement_processing(
        &self,
        actor: String
    ) -> Result<usize, crate::modules::settlement::errors::SettlementError> {
        let count = self.settlement_use_case.process_pending().await?;
        self.audit.log(LogAction {
            action: "admin.settlement_triggered",
            actor,
            entity_type: "settlement",
            entity_id: Uuid::new_v4(),
            merchant_id: None,
            metadata: Some(serde_json::json!({ "processed": count })),
            ip_address: None,
        }).await;
        info!(count, "Settlement processing triggered by admin");
        Ok(count)
    }

    pub async fn list_pending_webhook_events(
        &self
    ) -> Result<Vec<WebhookEvent>, crate::modules::webhook::errors::WebhookError> {
        self.webhook_repo.find_pending_events().await
    }

    pub async fn retry_webhooks(
        &self,
        actor: String
    ) -> Result<(), crate::modules::webhook::errors::WebhookError> {
        self.webhook_use_case.dispatch_pending().await?;
        self.audit.log(LogAction {
            action: "admin.webhook_retried",
            actor,
            entity_type: "webhook_event",
            entity_id: Uuid::new_v4(),
            merchant_id: None,
            metadata: None,
            ip_address: None,
        }).await;
        Ok(())
    }

    pub async fn get_recent_audit_logs(&self, limit: i64) -> Vec<super::audit::AuditLog> {
        self.audit.find_recent(limit).await
    }

    pub async fn get_merchant_audit_logs(
        &self,
        merchant_id: Uuid,
        limit: i64
    ) -> Vec<super::audit::AuditLog> {
        self.audit.find_by_merchant(merchant_id, limit).await
    }
}
