use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use super::errors::WalletError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Blockchain {
    Solana,
    Ethereum,
    Polygon,
    Base,
    Bsc,
    Tron,
    Bitcoin,
}

impl Blockchain {
    pub fn as_str(&self) -> &str {
        match self {
            Blockchain::Solana => "solana",
            Blockchain::Ethereum => "ethereum",
            Blockchain::Polygon => "polygon",
            Blockchain::Base => "base",
            Blockchain::Bsc => "bsc",
            Blockchain::Tron => "tron",
            Blockchain::Bitcoin => "bitcoin",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, WalletError> {
        match s {
            "solana" => Ok(Blockchain::Solana),
            "ethereum" => Ok(Blockchain::Ethereum),
            "polygon" => Ok(Blockchain::Polygon),
            "base" => Ok(Blockchain::Base),
            "bsc" => Ok(Blockchain::Bsc),
            "tron" => Ok(Blockchain::Tron),
            "bitcoin" => Ok(Blockchain::Bitcoin),
            other => Err(WalletError::UnsupportedBlockchain(other.to_string())),
        }
    }

    /// Returns true if the chain uses EVM address format (0x hex).
    #[allow(dead_code)]
    pub fn is_evm(&self) -> bool {
        matches!(self, Self::Ethereum | Self::Polygon | Self::Base | Self::Bsc)
    }

    /// Returns true if the chain uses Tron Base58Check addresses (starts with T).
    #[allow(dead_code)]
    pub fn is_tron(&self) -> bool {
        matches!(self, Self::Tron)
    }

    /// Returns true if the chain uses Solana Base58 addresses (32-byte pubkey).
    #[allow(dead_code)]
    pub fn is_solana(&self) -> bool {
        matches!(self, Self::Solana)
    }

    /// Returns true if the chain is Bitcoin.
    #[allow(dead_code)]
    pub fn is_bitcoin(&self) -> bool {
        matches!(self, Self::Bitcoin)
    }
}

#[derive(Debug, Clone)]
pub enum Asset {
    Usdc,
    Usdt, // TRC-20 USDT on Tron, BEP-20 USDT on BSC
    Sol,
    Eth,
    Bnb,  // native BNB on BSC
    Trx,  // native TRX on Tron
    Btc,  // native BTC on Bitcoin
}

impl Asset {
    pub fn as_str(&self) -> &str {
        match self {
            Asset::Usdc => "USDC",
            Asset::Usdt => "USDT",
            Asset::Sol => "SOL",
            Asset::Eth => "ETH",
            Asset::Bnb => "BNB",
            Asset::Trx => "TRX",
            Asset::Btc => "BTC",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, WalletError> {
        match s {
            "USDC" => Ok(Asset::Usdc),
            "USDT" => Ok(Asset::Usdt),
            "SOL" => Ok(Asset::Sol),
            "ETH" => Ok(Asset::Eth),
            "BNB" => Ok(Asset::Bnb),
            "TRX" => Ok(Asset::Trx),
            "BTC" => Ok(Asset::Btc),
            other => Err(WalletError::UnsupportedAsset(other.to_string())),
        }
    }

    /// Number of decimal places for this asset.
    #[allow(dead_code)]
    pub fn decimals(&self) -> u8 {
        match self {
            Self::Usdc => 6,
            Self::Usdt => 6,
            Self::Sol => 9,
            Self::Eth => 18,
            Self::Bnb => 18,
            Self::Trx => 6,  // 1 TRX = 1_000_000 sun
            Self::Btc => 8,  // 1 BTC = 100_000_000 satoshi
        }
    }
}

/// Validates a wallet address format for the given blockchain.
pub fn validate_wallet_address(blockchain: &Blockchain, address: &str) -> Result<(), WalletError> {
    match blockchain {
        Blockchain::Ethereum | Blockchain::Polygon | Blockchain::Base | Blockchain::Bsc => {
            validate_evm_address(address)
        }
        Blockchain::Solana => validate_solana_address(address),
        Blockchain::Tron => validate_tron_address(address),
        Blockchain::Bitcoin => validate_bitcoin_address(address),
    }
}

fn validate_evm_address(address: &str) -> Result<(), WalletError> {
    if !address.starts_with("0x") {
        return Err(WalletError::InvalidAddress);
    }
    let hex_part = &address[2..];
    if hex_part.len() != 40 {
        return Err(WalletError::InvalidAddress);
    }
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(WalletError::InvalidAddress);
    }
    Ok(())
}

fn validate_solana_address(address: &str) -> Result<(), WalletError> {
    // Solana addresses are Base58-encoded 32-byte public keys (32-44 chars)
    if address.len() < 32 || address.len() > 44 {
        return Err(WalletError::InvalidAddress);
    }
    let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| valid_chars.contains(c)) {
        return Err(WalletError::InvalidAddress);
    }
    Ok(())
}

fn validate_tron_address(address: &str) -> Result<(), WalletError> {
    // Tron addresses are Base58Check-encoded, always start with 'T', length 34
    if !address.starts_with('T') {
        return Err(WalletError::InvalidAddress);
    }
    if address.len() != 34 {
        return Err(WalletError::InvalidAddress);
    }
    let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| valid_chars.contains(c)) {
        return Err(WalletError::InvalidAddress);
    }
    Ok(())
}

fn validate_bitcoin_address(address: &str) -> Result<(), WalletError> {
    // P2PKH — starts with '1', Base58Check, 25-34 chars
    if address.starts_with('1') {
        return validate_base58_bitcoin(address, 25, 34);
    }

    // P2SH — starts with '3', Base58Check, 25-34 chars
    if address.starts_with('3') {
        return validate_base58_bitcoin(address, 25, 34);
    }

    // Bech32 / Bech32m — starts with "bc1" (mainnet), lowercase only
    if address.starts_with("bc1") {
        return validate_bech32_bitcoin(address);
    }

    Err(WalletError::InvalidAddress)
}

fn validate_base58_bitcoin(address: &str, min_len: usize, max_len: usize) -> Result<(), WalletError> {
    let len = address.len();
    if len < min_len || len > max_len {
        return Err(WalletError::InvalidAddress);
    }
    // Bitcoin Base58 alphabet (no 0, O, I, l)
    let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| valid_chars.contains(c)) {
        return Err(WalletError::InvalidAddress);
    }
    Ok(())
}

fn validate_bech32_bitcoin(address: &str) -> Result<(), WalletError> {
    // bc1q... (P2WPKH, 42 chars) or bc1p... (P2TR Taproot, 62 chars)
    // or bc1... segwit v0 (42) / v1 (62)
    let lower = address.to_lowercase();
    if !lower.starts_with("bc1") {
        return Err(WalletError::InvalidAddress);
    }
    let len = lower.len();
    if !(42..=90).contains(&len) {
        return Err(WalletError::InvalidAddress);
    }
    // Bech32 charset: qpzry9x8gf2tvdw0s3jn54khce6mua7l
    let valid_chars = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let data_part = &lower[3..]; // after "bc1"
    if !data_part.chars().all(|c| valid_chars.contains(c)) {
        return Err(WalletError::InvalidAddress);
    }
    Ok(())
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
    pub fn new(
        merchant_id: Uuid,
        address: String,
        blockchain: Blockchain,
        asset: Asset,
    ) -> Result<Self, WalletError> {
        let address = address.trim().to_string();
        validate_wallet_address(&blockchain, &address)?;

        Ok(Self {
            id: Uuid::new_v4(),
            merchant_id,
            address,
            blockchain,
            asset,
            balance: Decimal::ZERO,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Create a wallet from a generated keypair (no address validation needed —
    /// address was derived from the private key so it is always valid).
    pub fn from_generated(
        merchant_id: Uuid,
        address: String,
        blockchain: Blockchain,
        asset: Asset,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merchant_id,
            address,
            blockchain,
            asset,
            balance: Decimal::ZERO,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}