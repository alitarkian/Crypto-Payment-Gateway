use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::modules::wallet::{
    domain::{Asset, Blockchain, Wallet},
    errors::WalletError,
    repository::WalletRepository,
};

pub struct PostgresWalletRepository {
    pool: PgPool,
}

impl PostgresWalletRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

struct WalletRow {
    id: Uuid,
    merchant_id: Uuid,
    address: String,
    blockchain: String,
    asset: String,
    balance: Decimal,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WalletRow {
    fn into_domain(self) -> Result<Wallet, WalletError> {
        Ok(Wallet {
            id: self.id,
            merchant_id: self.merchant_id,
            address: self.address,
            blockchain: Blockchain::from_str(&self.blockchain)?,
            asset: Asset::from_str(&self.asset)?,
            balance: self.balance,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl WalletRepository for PostgresWalletRepository {
    async fn save(&self, wallet: &Wallet) -> Result<(), WalletError> {
        sqlx::query!(
            r#"
            INSERT INTO wallets (id, merchant_id, address, blockchain, asset, balance, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            wallet.id,
            wallet.merchant_id,
            wallet.address,
            wallet.blockchain.as_str(),
            wallet.asset.as_str(),
            wallet.balance,
            wallet.is_active,
            wallet.created_at,
            wallet.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err) if db_err.constraint() == Some("wallets_address_key") => {
                WalletError::AddressAlreadyExists
            }
            _ => WalletError::DatabaseError(e.to_string()),
        })?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Wallet, WalletError> {
        sqlx::query_as!(
            WalletRow,
            r#"SELECT id, merchant_id, address, blockchain, asset, balance, is_active, created_at, updated_at
               FROM wallets WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?
        .ok_or(WalletError::NotFound)?
        .into_domain()
    }

    async fn find_by_merchant_id(&self, merchant_id: Uuid) -> Result<Vec<Wallet>, WalletError> {
        sqlx::query_as!(
            WalletRow,
            r#"SELECT id, merchant_id, address, blockchain, asset, balance, is_active, created_at, updated_at
               FROM wallets WHERE merchant_id = $1"#,
            merchant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?
        .into_iter()
        .map(WalletRow::into_domain)
        .collect()
    }

    async fn find_by_address(&self, address: &str) -> Result<Option<Wallet>, WalletError> {
        sqlx::query_as!(
            WalletRow,
            r#"SELECT id, merchant_id, address, blockchain, asset, balance, is_active, created_at, updated_at
               FROM wallets WHERE address = $1"#,
            address
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| WalletError::DatabaseError(e.to_string()))?
        .map(WalletRow::into_domain)
        .transpose()
    }
}