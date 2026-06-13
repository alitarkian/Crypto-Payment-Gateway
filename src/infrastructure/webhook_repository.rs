use async_trait::async_trait;
use chrono::{ DateTime, Utc };
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::webhook::{
    domain::{ Webhook, WebhookDelivery, WebhookEvent, WebhookEventStatus, WebhookEventType },
    errors::WebhookError,
    repository::WebhookRepository,
};

pub struct PostgresWebhookRepository {
    pool: PgPool,
}

impl PostgresWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ─── Row types ───────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct WebhookRow {
    id: Uuid,
    merchant_id: Uuid,
    url: String,
    secret: String,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WebhookRow {
    fn into_domain(self) -> Webhook {
        Webhook {
            id: self.id,
            merchant_id: self.merchant_id,
            url: self.url,
            secret: self.secret,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct WebhookEventRow {
    id: Uuid,
    merchant_id: Uuid,
    event_type: String,
    payload: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WebhookEventRow {
    fn into_domain(self) -> Result<WebhookEvent, WebhookError> {
        let event_type = WebhookEventType::from_str(&self.event_type).ok_or_else(||
            WebhookError::DatabaseError(format!("Unknown event type: {}", self.event_type))
        )?;

        Ok(WebhookEvent {
            id: self.id,
            merchant_id: self.merchant_id,
            event_type,
            payload: self.payload,
            status: WebhookEventStatus::from_str(&self.status),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

// ─── Implementation ──────────────────────────────────────────────────────────

#[async_trait]
impl WebhookRepository for PostgresWebhookRepository {
    async fn save_webhook(&self, webhook: &Webhook) -> Result<(), WebhookError> {
        sqlx
            ::query(
                r#"
            INSERT INTO webhooks (id, merchant_id, url, secret, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
            )
            .bind(webhook.id)
            .bind(webhook.merchant_id)
            .bind(&webhook.url)
            .bind(&webhook.secret)
            .bind(webhook.is_active)
            .bind(webhook.created_at)
            .bind(webhook.updated_at)
            .execute(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_webhook_by_id(&self, id: Uuid) -> Result<Webhook, WebhookError> {
        sqlx::query_as::<_, WebhookRow>(
            "SELECT id, merchant_id, url, secret, is_active, created_at, updated_at
             FROM webhooks WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?
            .ok_or(WebhookError::NotFound)
            .map(WebhookRow::into_domain)
    }

    async fn find_webhooks_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Webhook>, WebhookError> {
        sqlx::query_as::<_, WebhookRow>(
            "SELECT id, merchant_id, url, secret, is_active, created_at, updated_at
             FROM webhooks WHERE merchant_id = $1 AND is_active = true"
        )
            .bind(merchant_id)
            .fetch_all(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))
            .map(|rows| rows.into_iter().map(WebhookRow::into_domain).collect())
    }

    async fn save_event(&self, event: &WebhookEvent) -> Result<(), WebhookError> {
        sqlx
            ::query(
                r#"
            INSERT INTO webhook_events (id, merchant_id, event_type, payload, status, created_at, updated_at)
            VALUES ($1, $2, $3::text::webhook_event_type, $4, $5::text::webhook_event_status, $6, $7)
            "#
            )
            .bind(event.id)
            .bind(event.merchant_id)
            .bind(event.event_type.as_str())
            .bind(&event.payload)
            .bind(event.status.as_str())
            .bind(event.created_at)
            .bind(event.updated_at)
            .execute(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_event(&self, event: &WebhookEvent) -> Result<(), WebhookError> {
        sqlx
            ::query(
                r#"
            UPDATE webhook_events
            SET status = $1::text::webhook_event_status, updated_at = $2
            WHERE id = $3
            "#
            )
            .bind(event.status.as_str())
            .bind(event.updated_at)
            .bind(event.id)
            .execute(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_pending_events(&self) -> Result<Vec<WebhookEvent>, WebhookError> {
        sqlx::query_as::<_, WebhookEventRow>(
            r#"
            SELECT id, merchant_id, event_type::TEXT as event_type,
                   payload, status::TEXT as status, created_at, updated_at
            FROM webhook_events
            WHERE status = 'pending'
            ORDER BY created_at ASC
            LIMIT 100
            "#
        )
            .fetch_all(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(WebhookEventRow::into_domain)
            .collect()
    }

    async fn save_delivery(&self, delivery: &WebhookDelivery) -> Result<(), WebhookError> {
        sqlx
            ::query(
                r#"
            INSERT INTO webhook_deliveries
                (id, webhook_event_id, webhook_id, attempt, status_code, response_body,
                 error_message, delivered_at, next_retry_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
            )
            .bind(delivery.id)
            .bind(delivery.webhook_event_id)
            .bind(delivery.webhook_id)
            .bind(delivery.attempt)
            .bind(delivery.status_code)
            .bind(&delivery.response_body)
            .bind(&delivery.error_message)
            .bind(delivery.delivered_at)
            .bind(delivery.next_retry_at)
            .bind(delivery.created_at)
            .execute(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn count_deliveries_for_event(&self, event_id: Uuid) -> Result<i64, WebhookError> {
        let row: (i64,) = sqlx
            ::query_as("SELECT COUNT(*) FROM webhook_deliveries WHERE webhook_event_id = $1")
            .bind(event_id)
            .fetch_one(&self.pool).await
            .map_err(|e| WebhookError::DatabaseError(e.to_string()))?;

        Ok(row.0)
    }
}
