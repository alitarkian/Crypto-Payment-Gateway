use chrono::{DateTime, Utc};
use uuid::Uuid;
use super::errors::MerchantError;

#[derive(Debug, Clone)]
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

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

fn generate_api_key() -> String {
    use std::fmt::Write;
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let mut key = String::with_capacity(64);
    write!(key, "mk_{}{}", id1.simple(), id2.simple()).unwrap();
    key
}