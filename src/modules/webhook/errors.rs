use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[allow(dead_code)]
    #[error("Webhook not found")]
    NotFound,

    #[allow(dead_code)]
    #[error("Webhook event not found")]
    EventNotFound,

    #[allow(dead_code)]
    #[error("Merchant not found")]
    MerchantNotFound,

    #[allow(dead_code)]
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
}