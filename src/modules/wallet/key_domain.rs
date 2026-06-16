//! WalletKey — domain entity for encrypted private keys.
//!
//! This entity NEVER leaves the infrastructure layer.
//! It is never serialized to JSON, never returned in API responses,
//! and never written to logs.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// An encrypted private key associated with a managed wallet.
///
/// The private key bytes are stored as AES-256-GCM ciphertext.
/// Only `WalletVault` knows how to decrypt them.
#[derive(Debug, Clone)]
pub struct WalletKey {
    pub id: Uuid,
    pub wallet_id: Uuid,
    /// AES-256-GCM ciphertext of the raw private key bytes
    pub encrypted_private_key: Vec<u8>,
    /// 12-byte GCM nonce used during encryption
    pub key_nonce: Vec<u8>,
    /// Master key version used for this encryption (for key rotation)
    pub key_version: u32,
    pub created_at: DateTime<Utc>,
}

impl WalletKey {
    pub fn new(
        wallet_id: Uuid,
        encrypted_private_key: Vec<u8>,
        key_nonce: Vec<u8>,
        key_version: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            encrypted_private_key,
            key_nonce,
            key_version,
            created_at: Utc::now(),
        }
    }
}
