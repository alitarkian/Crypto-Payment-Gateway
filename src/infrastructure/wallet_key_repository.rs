use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::modules::wallet::{
    errors::WalletError,
    key_domain::WalletKey,
    key_repository::WalletKeyRepository,
};

pub struct PostgresWalletKeyRepository {
    pool: PgPool,
}

impl PostgresWalletKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct WalletKeyRow {
    id: Uuid,
    wallet_id: Uuid,
    encrypted_private_key: Vec<u8>,
    key_nonce: Vec<u8>,
    key_version: i32,
    created_at: DateTime<Utc>,
}

impl WalletKeyRow {
    fn into_domain(self) -> WalletKey {
        WalletKey {
            id: self.id,
            wallet_id: self.wallet_id,
            encrypted_private_key: self.encrypted_private_key,
            key_nonce: self.key_nonce,
            key_version: self.key_version as u32,
            created_at: self.created_at,
        }
    }
}

#[async_trait]
impl WalletKeyRepository for PostgresWalletKeyRepository {
    async fn save(&self, key: &WalletKey) -> Result<(), WalletError> {
        sqlx::query(
            r#"
            INSERT INTO wallet_keys
                (id, wallet_id, encrypted_private_key, key_nonce, key_version, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(key.id)
        .bind(key.wallet_id)
        .bind(&key.encrypted_private_key)
        .bind(&key.key_nonce)
        .bind(key.key_version as i32)
        .bind(key.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<WalletKey, WalletError> {
        sqlx::query_as::<_, WalletKeyRow>(
            r#"
            SELECT id, wallet_id, encrypted_private_key, key_nonce, key_version, created_at
            FROM wallet_keys
            WHERE wallet_id = $1
            "#,
        )
        .bind(wallet_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?
        .ok_or(WalletError::NotFound)
        .map(WalletKeyRow::into_domain)
    }
}
