use thiserror::Error;

#[derive(Debug, Error)]
pub enum MerchantError {
    #[error("Merchant not found")]
    NotFound,

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email address")]
    InvalidEmail,

    #[error("Merchant name cannot be empty")]
    EmptyName,

    #[error("Database error: {0}")]
    DatabaseError(String),
}