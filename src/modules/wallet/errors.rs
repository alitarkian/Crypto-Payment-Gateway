use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Wallet not found")]
    NotFound,

    #[error("Merchant not found")]
    MerchantNotFound,

    #[error("Address already exists")]
    AddressAlreadyExists,

    #[error("Invalid blockchain address")]
    InvalidAddress,

    #[error("Wallet is inactive")]
    InactiveWallet,

    #[error("Database error: {0}")]
    DatabaseError(String),
}