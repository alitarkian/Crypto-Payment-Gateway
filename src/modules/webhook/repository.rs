use async_trait::async_trait;
use uuid::Uuid;

use super::domain::{Webhook, WebhookDelivery, WebhookEvent};
use super::errors::WebhookError;

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn save_webhook(&self, webhook: &Webhook) -> Result<(), WebhookError>;
    async fn find_webhook_by_id(&self, id: Uuid) -> Result<Webhook, WebhookError>;
    async fn find_webhooks_by_merchant(&self, merchant_id: Uuid) -> Result<Vec<Webhook>, WebhookError>;

    async fn save_event(&self, event: &WebhookEvent) -> Result<(), WebhookError>;
    async fn update_event(&self, event: &WebhookEvent) -> Result<(), WebhookError>;
    async fn find_pending_events(&self) -> Result<Vec<WebhookEvent>, WebhookError>;

    async fn save_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError>;
    async fn count_deliveries_for_event(&self, event_id: Uuid) -> Result<i64, WebhookError>;
}