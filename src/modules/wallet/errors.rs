use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Wallet not found")]
    NotFound,

    #[allow(dead_code)]
    #[error("Merchant not found")]
    MerchantNotFound,

    #[error("Address already exists")]
    AddressAlreadyExists,

    #[error("Invalid blockchain address")]
    InvalidAddress,

    #[allow(dead_code)]
    #[error("Wallet is inactive")]
    InactiveWallet,

    #[error("Database error: {0}")]
    DatabaseError(String),
}