use chrono::{ DateTime, Utc };
use uuid::Uuid;
use super::errors::MerchantError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Merchant {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Merchant {
    pub fn new(name: String, email: String) -> Result<Self, MerchantError> {
        if name.trim().is_empty() {
            return Err(MerchantError::EmptyName);
        }

        if !email.contains('@') {
            return Err(MerchantError::InvalidEmail);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            email: email.trim().to_lowercase(),
            api_key: generate_api_key(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    // pub fn is_active(&self) -> bool {
    //     self.is_active
    // }
}

/// Generates a cryptographically secure API key using OS entropy.
///
/// Format: `cgpk_<64-char lowercase hex>` (256 bits of OsRng entropy)
/// Total length: 69 characters.
fn generate_api_key() -> String {
    use rand::RngCore;
    use rand::rngs::OsRng;

    let mut bytes = [0u8; 32]; // 256 bits
    OsRng.fill_bytes(&mut bytes);

    format!("cgpk_{}", hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Merchant::new() ───────────────────────────────────────────────────────

    #[test]
    fn valid_merchant_created() {
        let m = Merchant::new("Acme Corp".to_string(), "admin@acme.com".to_string()).unwrap();
        assert_eq!(m.name, "Acme Corp");
        assert_eq!(m.email, "admin@acme.com");
        assert!(m.is_active);
    }

    #[test]
    fn empty_name_returns_err() {
        let result = Merchant::new("   ".to_string(), "test@test.com".to_string());
        assert!(matches!(result, Err(MerchantError::EmptyName)));
    }

    #[test]
    fn invalid_email_returns_err() {
        let result = Merchant::new("Acme".to_string(), "notanemail".to_string());
        assert!(matches!(result, Err(MerchantError::InvalidEmail)));
    }

    #[test]
    fn email_is_lowercased() {
        let m = Merchant::new("Acme".to_string(), "ADMIN@ACME.COM".to_string()).unwrap();
        assert_eq!(m.email, "admin@acme.com");
    }

    // ── API Key (D5) ──────────────────────────────────────────────────────────

    #[test]
    fn api_key_has_correct_prefix() {
        let key = generate_api_key();
        assert!(key.starts_with("cgpk_"), "Key must start with 'cgpk_', got: {}", key);
    }

    #[test]
    fn api_key_has_correct_length() {
        let key = generate_api_key();
        // "cgpk_" (5) + 64 hex chars = 69
        assert_eq!(key.len(), 69, "Expected 69 chars, got {}: {}", key.len(), key);
    }

    #[test]
    fn api_key_suffix_is_valid_hex() {
        let key = generate_api_key();
        let hex_part = &key[5..]; // strip "cgpk_"
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Key suffix is not valid hex: {}",
            hex_part
        );
    }

    #[test]
    fn api_keys_are_unique() {
        let k1 = generate_api_key();
        let k2 = generate_api_key();
        assert_ne!(k1, k2, "Two generated keys must not be equal");
    }
}
