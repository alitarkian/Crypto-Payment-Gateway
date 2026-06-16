//! GenerateWallet use case — creates a new managed (custodial) wallet.
//!
//! Keypair generation per chain:
//!   - Solana       → ed25519-dalek  (Base58 address from 32-byte pubkey)
//!   - EVM chains   → secp256k1      (0x + keccak256 last 20 bytes of uncompressed pubkey)
//!   - Tron         → secp256k1      (Base58Check, same derivation as EVM but different prefix)
//!   - Bitcoin      → secp256k1      (P2PKH: Base58Check with version byte 0x00)
//!
//! Security invariants:
//!   - Private key bytes are zeroized immediately after encryption.
//!   - Private key is NEVER returned in the result.
//!   - Every generation is saved atomically: Wallet + WalletKey in the same logical operation.

use std::sync::Arc;
use tracing::info;
use uuid::Uuid;
use zeroize::Zeroizing;

use super::{
    domain::{Asset, Blockchain, Wallet},
    errors::WalletError,
    key_domain::WalletKey,
    key_repository::WalletKeyRepository,
    repository::WalletRepository,
};
use crate::infrastructure::vault::key_vault::WalletVault;

pub struct GenerateWallet {
    pub merchant_id: Uuid,
    pub blockchain: Blockchain,
    pub asset: Asset,
}

pub struct GeneratedWallet {
    pub wallet: Wallet,
    // private key intentionally NOT included
}

pub struct GenerateWalletUseCase {
    wallet_repo: Arc<dyn WalletRepository>,
    key_repo: Arc<dyn WalletKeyRepository>,
    vault: Arc<WalletVault>,
}

impl GenerateWalletUseCase {
    pub fn new(
        wallet_repo: Arc<dyn WalletRepository>,
        key_repo: Arc<dyn WalletKeyRepository>,
        vault: Arc<WalletVault>,
    ) -> Self {
        Self { wallet_repo, key_repo, vault }
    }

    pub async fn execute(&self, cmd: GenerateWallet) -> Result<GeneratedWallet, WalletError> {
        let (address, mut raw_key) = match cmd.blockchain {
            Blockchain::Solana => generate_solana_keypair()?,
            Blockchain::Ethereum
            | Blockchain::Polygon
            | Blockchain::Base
            | Blockchain::Bsc => generate_evm_keypair()?,
            Blockchain::Tron => generate_tron_keypair()?,
            Blockchain::Bitcoin => generate_bitcoin_p2pkh_keypair()?,
        };

        // Encrypt the private key — raw_key is zeroized inside encrypt()
        let (ciphertext, nonce) = self.vault.encrypt(&mut raw_key)?;

        // Create wallet (skip address validation — address was derived from key)
        let wallet = Wallet::from_generated(
            cmd.merchant_id,
            address,
            cmd.blockchain,
            cmd.asset,
        );

        // Persist wallet
        self.wallet_repo.save(&wallet).await?;

        // Persist encrypted key
        let wallet_key = WalletKey::new(
            wallet.id,
            ciphertext,
            nonce,
            self.vault.key_version,
        );
        self.key_repo.save(&wallet_key).await?;

        info!(
            wallet_id = %wallet.id,
            merchant_id = %wallet.merchant_id,
            blockchain = wallet.blockchain.as_str(),
            "Managed wallet generated"
        );

        Ok(GeneratedWallet { wallet })
    }
}

// ── Key generation helpers ────────────────────────────────────────────────────

fn generate_solana_keypair() -> Result<(String, Zeroizing<Vec<u8>>), WalletError> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    // Solana address = Base58 of the 32-byte public key
    let address = bs58::encode(verifying_key.as_bytes()).into_string();

    // Private key = 32-byte seed (scalar)
    let raw_key = Zeroizing::new(signing_key.to_bytes().to_vec());

    Ok((address, raw_key))
}

fn generate_evm_keypair() -> Result<(String, Zeroizing<Vec<u8>>), WalletError> {
    use secp256k1::{Secp256k1, SecretKey};
    use rand::rngs::OsRng;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut OsRng);
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

    // EVM address = keccak256(uncompressed_pubkey[1..])[12..]  (last 20 bytes)
    let uncompressed = public_key.serialize_uncompressed(); // 65 bytes, starts with 0x04
    let hash = keccak256(&uncompressed[1..]); // skip the 0x04 prefix
    let address = format!("0x{}", hex::encode(&hash[12..]));

    let raw_key = Zeroizing::new(secret_key[..].to_vec());

    Ok((address, raw_key))
}

fn generate_tron_keypair() -> Result<(String, Zeroizing<Vec<u8>>), WalletError> {
    use secp256k1::{Secp256k1, SecretKey};
    use rand::rngs::OsRng;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut OsRng);
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

    // Tron address derivation:
    //   1. keccak256(uncompressed_pubkey[1..])[12..] → 20 bytes (same as EVM)
    //   2. Prepend 0x41 (Tron mainnet version byte)
    //   3. Base58Check encode
    let uncompressed = public_key.serialize_uncompressed();
    let hash = keccak256(&uncompressed[1..]);
    let payload_20 = &hash[12..]; // 20 bytes

    let mut raw_address = vec![0x41u8]; // Tron mainnet prefix
    raw_address.extend_from_slice(payload_20);

    let address = base58check_encode(&raw_address);
    let raw_key = Zeroizing::new(secret_key[..].to_vec());

    Ok((address, raw_key))
}

fn generate_bitcoin_p2pkh_keypair() -> Result<(String, Zeroizing<Vec<u8>>), WalletError> {
    use secp256k1::{Secp256k1, SecretKey};
    use rand::rngs::OsRng;

    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut OsRng);
    let public_key = secp256k1::PublicKey::from_secret_key(&secp, &secret_key);

    // Bitcoin P2PKH address derivation:
    //   1. SHA256(compressed_pubkey)
    //   2. RIPEMD160(result)             → pubkey_hash (20 bytes)
    //   3. Prepend 0x00 (mainnet version byte)
    //   4. Base58Check encode
    let compressed = public_key.serialize(); // 33 bytes
    let sha256_hash = {
        use sha2::{Digest, Sha256};
        Sha256::digest(&compressed)
    };

    use ripemd::Digest as RipemdDigest;
    let pubkey_hash = ripemd::Ripemd160::digest(sha256_hash);

    let mut payload = vec![0x00u8]; // mainnet P2PKH version
    payload.extend_from_slice(&pubkey_hash);

    let address = base58check_encode(&payload);
    let raw_key = Zeroizing::new(secret_key[..].to_vec());

    Ok((address, raw_key))
}

// ── Crypto primitives ─────────────────────────────────────────────────────────

fn keccak256(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Keccak256};
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Base58Check encoding: payload + 4-byte checksum (double-SHA256 of payload).
fn base58check_encode(payload: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);
    let checksum = &hash2[..4];

    let mut full = payload.to_vec();
    full.extend_from_slice(checksum);

    bs58::encode(full).into_string()
}
