use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvoiceError {
    #[error("Invoice not found")]
    NotFound,

    #[error("Invoice already paid")]
    AlreadyPaid,

    #[error("Invoice already expired")]
    AlreadyExpired,

    #[error("Invoice already cancelled")]
    AlreadyCancelled,

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Invalid expiration: must be in the future")]
    InvalidExpiration,

    #[error("Database error: {0}")]
    DatabaseError(String),
}