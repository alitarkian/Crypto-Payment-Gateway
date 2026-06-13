use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use super::errors::WalletError;

#[derive(Debug, Clone)]
pub enum Blockchain {
    Solana,
}

impl Blockchain {
    pub fn as_str(&self) -> &str {
        match self {
            Blockchain::Solana => "solana",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, WalletError> {
        match s {
            "solana" => Ok(Blockchain::Solana),
            _ => Err(WalletError::InvalidAddress),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Asset {
    USDC,
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Asset::USDC => "USDC",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, WalletError> {
        match s {
            "USDC" => Ok(Asset::USDC),
            _ => Err(WalletError::InvalidAddress),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wallet {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub address: String,
    pub blockchain: Blockchain,
    pub asset: Asset,
    pub balance: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Wallet {
    pub fn new(merchant_id: Uuid, address: String) -> Result<Self, WalletError> {
        if address.trim().is_empty() {
            return Err(WalletError::InvalidAddress);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            merchant_id,
            address: address.trim().to_string(),
            blockchain: Blockchain::Solana,
            asset: Asset::USDC,
            balance: Decimal::ZERO,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}