//! WalletVault — AES-256-GCM encryption for private keys at rest.
//!
//! Security invariants:
//!   - Master key is ONLY loaded from the `WALLET_MASTER_KEY` environment variable.
//!   - Master key must be exactly 32 bytes (64 hex chars).
//!   - A fresh random 12-byte nonce is generated for every encryption operation.
//!   - Private key bytes are zeroized from memory immediately after use.
//!   - Decrypted key bytes are returned as `ZeroizingBytes` — caller must drop promptly.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use zeroize::{Zeroize, Zeroizing};

use crate::modules::wallet::errors::WalletError;

/// A newtype around `Vec<u8>` that zeroizes on drop.
/// Used to hold decrypted private key bytes safely.
pub type ZeroizingBytes = Zeroizing<Vec<u8>>;

pub struct WalletVault {
    cipher: Aes256Gcm,
    /// Key version — increment when rotating master key.
    pub key_version: u32,
}

impl WalletVault {
    /// Load the vault from the `WALLET_MASTER_KEY` environment variable.
    ///
    /// `WALLET_MASTER_KEY` must be a 64-character lowercase hex string (32 bytes).
    /// Returns an error if the env var is absent, not valid hex, or wrong length.
    pub fn from_env() -> Result<Self, WalletError> {
        let hex_key = std::env::var("WALLET_MASTER_KEY")
            .map_err(|_| WalletError::EncryptionError(
                "WALLET_MASTER_KEY environment variable is not set".to_string(),
            ))?;

        let key_bytes = hex::decode(hex_key.trim())
            .map_err(|_| WalletError::EncryptionError(
                "WALLET_MASTER_KEY is not valid hex".to_string(),
            ))?;

        if key_bytes.len() != 32 {
            return Err(WalletError::EncryptionError(format!(
                "WALLET_MASTER_KEY must be 32 bytes (64 hex chars), got {}",
                key_bytes.len()
            )));
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher, key_version: 1 })
    }

    /// Encrypt raw private key bytes.
    ///
    /// Returns `(ciphertext, nonce)` — both must be stored together in the DB.
    /// The input `plaintext` is zeroized after encryption.
    pub fn encrypt(
        &self,
        plaintext: &mut ZeroizingBytes,
    ) -> Result<(Vec<u8>, Vec<u8>), WalletError> {
        // Generate a fresh random 12-byte nonce for every encryption
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = self.cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| WalletError::EncryptionError(format!("AES-GCM encrypt failed: {e}")))?;

        // Zeroize plaintext from memory now that we have the ciphertext
        plaintext.zeroize();

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypt ciphertext back to raw private key bytes.
    ///
    /// Returns `ZeroizingBytes` — caller must drop this as soon as possible
    /// to minimize the window where the key exists in plaintext in memory.
    pub fn decrypt(
        &self,
        ciphertext: &[u8],
        nonce_bytes: &[u8],
    ) -> Result<ZeroizingBytes, WalletError> {
        if nonce_bytes.len() != 12 {
            return Err(WalletError::EncryptionError(
                "Invalid nonce length — expected 12 bytes".to_string(),
            ));
        }

        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = Nonce::from(nonce_arr);

        let plaintext = self.cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| WalletError::EncryptionError(
                "AES-GCM decryption failed — key or nonce mismatch".to_string(),
            ))?;

        Ok(Zeroizing::new(plaintext))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeroize::Zeroizing;

    fn test_vault() -> WalletVault {
        // 32 bytes of zeros — only for tests
        let key = Key::<Aes256Gcm>::from_slice(&[0u8; 32]);
        WalletVault {
            cipher: Aes256Gcm::new(key),
            key_version: 1,
        }
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let vault = test_vault();
        let secret = b"my_super_secret_private_key_bytes";

        let mut plaintext = Zeroizing::new(secret.to_vec());
        let (ciphertext, nonce) = vault.encrypt(&mut plaintext).unwrap();

        // After encrypt, plaintext should be zeroized
        assert!(plaintext.iter().all(|&b| b == 0));

        let decrypted = vault.decrypt(&ciphertext, &nonce).unwrap();
        assert_eq!(decrypted.as_ref(), secret);
    }

    #[test]
    fn wrong_nonce_returns_err() {
        let vault = test_vault();
        let mut plaintext = Zeroizing::new(b"test_key".to_vec());
        let (ciphertext, _) = vault.encrypt(&mut plaintext).unwrap();

        let bad_nonce = [0u8; 12];
        assert!(vault.decrypt(&ciphertext, &bad_nonce).is_err());
    }

    #[test]
    fn tampered_ciphertext_returns_err() {
        let vault = test_vault();
        let mut plaintext = Zeroizing::new(b"test_key_bytes_xx".to_vec());
        let (mut ciphertext, nonce) = vault.encrypt(&mut plaintext).unwrap();

        // Flip a byte
        ciphertext[0] ^= 0xFF;
        assert!(vault.decrypt(&ciphertext, &nonce).is_err());
    }

    #[test]
    fn unique_nonces_per_encrypt() {
        let vault = test_vault();
        let mut p1 = Zeroizing::new(b"key1".to_vec());
        let mut p2 = Zeroizing::new(b"key1".to_vec());

        let (_, nonce1) = vault.encrypt(&mut p1).unwrap();
        let (_, nonce2) = vault.encrypt(&mut p2).unwrap();

        // Two encryptions of the same plaintext must produce different nonces
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn from_env_rejects_wrong_length() {
        // SAFETY: test-only, single-threaded
        unsafe { std::env::set_var("WALLET_MASTER_KEY", "deadbeef"); } // too short
        let result = WalletVault::from_env();
        assert!(result.is_err());
        unsafe { std::env::remove_var("WALLET_MASTER_KEY"); }
    }
}
