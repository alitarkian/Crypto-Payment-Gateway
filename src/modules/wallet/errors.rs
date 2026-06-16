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

    #[error("Unsupported blockchain: {0}")]
    UnsupportedBlockchain(String),

    #[error("Unsupported asset: {0}")]
    UnsupportedAsset(String),

    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}