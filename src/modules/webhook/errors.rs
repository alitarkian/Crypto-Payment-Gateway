use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Webhook not found")]
    NotFound,

    #[error("Webhook event not found")]
    EventNotFound,

    #[error("Merchant not found")]
    MerchantNotFound,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
}