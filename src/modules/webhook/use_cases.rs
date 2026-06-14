use std::sync::Arc;
use tracing::{ error, info, warn };
use uuid::Uuid;

use super::domain::{ Webhook, WebhookDelivery, WebhookEvent, WebhookEventType };
use super::errors::WebhookError;
use super::repository::WebhookRepository;

const MAX_ATTEMPTS: i64 = 5;

#[allow(dead_code)]
pub struct RegisterWebhook {
    pub merchant_id: Uuid,
    pub url: String,
    pub secret: String,
}

pub struct CreateWebhookEvent {
    pub merchant_id: Uuid,
    pub event_type: WebhookEventType,
    pub payload: serde_json::Value,
}

pub struct WebhookUseCase {
    repo: Arc<dyn WebhookRepository>,
    http: reqwest::Client,
}

impl WebhookUseCase {
    pub fn new(repo: Arc<dyn WebhookRepository>) -> Self {
        Self {
            repo,
            http: reqwest::Client
                ::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    #[allow(dead_code)]
    pub async fn register(&self, cmd: RegisterWebhook) -> Result<Webhook, WebhookError> {
        let url = cmd.url.trim().to_string();
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(WebhookError::InvalidUrl(url));
        }

        let webhook = Webhook::new(cmd.merchant_id, url, cmd.secret);
        self.repo.save_webhook(&webhook).await?;

        info!(
            webhook_id = %webhook.id,
            merchant_id = %cmd.merchant_id,
            "Webhook registered"
        );

        Ok(webhook)
    }

    pub async fn create_event(
        &self,
        cmd: CreateWebhookEvent
    ) -> Result<WebhookEvent, WebhookError> {
        let event = WebhookEvent::new(cmd.merchant_id, cmd.event_type, cmd.payload);
        self.repo.save_event(&event).await?;

        info!(
            event_id = %event.id,
            merchant_id = %cmd.merchant_id,
            event_type = %event.event_type.as_str(),
            "Webhook event created"
        );

        Ok(event)
    }

    /// Called by background worker — delivers all pending events
    pub async fn dispatch_pending(&self) -> Result<(), WebhookError> {
        let events = self.repo.find_pending_events().await?;

        for mut event in events {
            let webhooks = self.repo.find_webhooks_by_merchant(event.merchant_id).await?;
            let active: Vec<_> = webhooks
                .into_iter()
                .filter(|w| w.is_active)
                .collect();

            if active.is_empty() {
                // No webhooks — mark failed so we don't loop forever
                event.mark_failed();
                self.repo.update_event(&event).await?;
                continue;
            }

            let attempt_count = self.repo.count_deliveries_for_event(event.id).await?;

            if attempt_count >= MAX_ATTEMPTS {
                warn!(event_id = %event.id, "Max delivery attempts reached");
                event.mark_failed();
                self.repo.update_event(&event).await?;
                continue;
            }

            let attempt = (attempt_count + 1) as i32;
            let mut all_ok = true;

            for webhook in &active {
                let mut delivery = WebhookDelivery::new(event.id, webhook.id, attempt);

                match self.deliver(&event, webhook).await {
                    Ok(status_code) => {
                        delivery.status_code = Some(status_code);
                        delivery.delivered_at = Some(chrono::Utc::now());
                        info!(
                            event_id = %event.id,
                            webhook_id = %webhook.id,
                            status_code,
                            "Webhook delivered"
                        );
                    }
                    Err(e) => {
                        all_ok = false;
                        delivery.error_message = Some(e.to_string());
                        delivery.next_retry_at = WebhookDelivery::next_retry(attempt);
                        warn!(
                            event_id = %event.id,
                            webhook_id = %webhook.id,
                            error = %e,
                            "Webhook delivery failed"
                        );
                    }
                }

                if let Err(e) = self.repo.save_delivery(&delivery).await {
                    error!(error = %e, "Failed to save delivery record");
                }
            }

            if all_ok {
                event.mark_delivered();
            } else if attempt >= (MAX_ATTEMPTS as i32) {
                event.mark_failed();
            }

            self.repo.update_event(&event).await?;
        }

        Ok(())
    }

    async fn deliver(&self, event: &WebhookEvent, webhook: &Webhook) -> Result<i32, WebhookError> {
        let signature = self.sign(&webhook.secret, &event.payload.to_string());

        let response = self.http
            .post(&webhook.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", signature)
            .header("X-Webhook-Event", event.event_type.as_str())
            .json(
                &serde_json::json!({
                "event_id": event.id,
                "event_type": event.event_type.as_str(),
                "merchant_id": event.merchant_id,
                "payload": event.payload,
                "created_at": event.created_at,
            })
            )
            .send().await
            .map_err(|e| WebhookError::DeliveryFailed(e.to_string()))?;

        let status = response.status().as_u16() as i32;

        if response.status().is_success() {
            Ok(status)
        } else {
            Err(WebhookError::DeliveryFailed(format!("HTTP {status}")))
        }
    }

    pub async fn list_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<super::domain::Webhook>, WebhookError> {
        self.repo.find_webhooks_by_merchant(merchant_id).await
    }

    fn sign(&self, secret: &str, payload: &str) -> String {
        use hmac::{ Hmac, Mac };
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect(
            "HMAC can take key of any size"
        );
        mac.update(payload.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }
}
