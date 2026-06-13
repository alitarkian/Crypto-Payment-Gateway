use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum WebhookEventType {
    InvoicePaid,
    InvoiceExpired,
    PaymentDetected,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &str {
        match self {
            WebhookEventType::InvoicePaid => "invoice.paid",
            WebhookEventType::InvoiceExpired => "invoice.expired",
            WebhookEventType::PaymentDetected => "payment.detected",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "invoice.paid" => Some(WebhookEventType::InvoicePaid),
            "invoice.expired" => Some(WebhookEventType::InvoiceExpired),
            "payment.detected" => Some(WebhookEventType::PaymentDetected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebhookEventStatus {
    Pending,
    Delivered,
    Failed,
}

impl WebhookEventStatus {
    pub fn as_str(&self) -> &str {
        match self {
            WebhookEventStatus::Pending => "pending",
            WebhookEventStatus::Delivered => "delivered",
            WebhookEventStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "delivered" => WebhookEventStatus::Delivered,
            "failed" => WebhookEventStatus::Failed,
            _ => WebhookEventStatus::Pending,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Webhook {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub url: String,
    pub secret: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Webhook {
    pub fn new(merchant_id: Uuid, url: String, secret: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merchant_id,
            url,
            secret,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub merchant_id: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub event_type: WebhookEventType,
    pub payload: serde_json::Value,
    pub status: WebhookEventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WebhookEvent {
    pub fn new(
        merchant_id: Uuid,
        event_type: WebhookEventType,
        payload: serde_json::Value,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merchant_id,
            event_type,
            payload,
            status: WebhookEventStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_delivered(&mut self) {
        self.status = WebhookEventStatus::Delivered;
        self.updated_at = Utc::now();
    }

    pub fn mark_failed(&mut self) {
        self.status = WebhookEventStatus::Failed;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_event_id: Uuid,
    pub webhook_id: Uuid,
    pub attempt: i32,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl WebhookDelivery {
    pub fn new(webhook_event_id: Uuid, webhook_id: Uuid, attempt: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            webhook_event_id,
            webhook_id,
            attempt,
            status_code: None,
            response_body: None,
            error_message: None,
            delivered_at: None,
            next_retry_at: None,
            created_at: Utc::now(),
        }
    }

    /// Exponential backoff: 1m, 5m, 30m, 2h, 8h
    pub fn next_retry(attempt: i32) -> Option<DateTime<Utc>> {
        let minutes: i64 = match attempt {
            1 => 1,
            2 => 5,
            3 => 30,
            4 => 120,
            5 => 480,
            _ => return None,
        };
        Some(Utc::now() + chrono::Duration::minutes(minutes))
    }
}