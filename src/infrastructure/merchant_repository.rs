use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use crate::modules::merchant::{
    domain::Merchant,
    errors::MerchantError,
    repository::MerchantRepository,
};

pub struct PostgresMerchantRepository {
    pool: PgPool,
}

impl PostgresMerchantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MerchantRepository for PostgresMerchantRepository {
    async fn save(&self, merchant: &Merchant) -> Result<(), MerchantError> {
        sqlx
            ::query(
                r#"INSERT INTO merchants (id, name, email, api_key, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#
            )
            .bind(merchant.id)
            .bind(&merchant.name)
            .bind(&merchant.email)
            .bind(&merchant.api_key)
            .bind(merchant.is_active)
            .bind(merchant.created_at)
            .bind(merchant.updated_at)
            .execute(&self.pool).await
            .map_err(|e| {
                match e {
                    sqlx::Error::Database(db_err) if
                        db_err.constraint() == Some("merchants_email_key")
                    => {
                        MerchantError::EmailAlreadyExists
                    }
                    _ => MerchantError::DatabaseError(e.to_string()),
                }
            })?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Merchant, MerchantError> {
        sqlx::query_as::<_, MerchantRow>(
            "SELECT id, name, email, api_key, is_active, created_at, updated_at
             FROM merchants WHERE id = $1"
        )
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| MerchantError::DatabaseError(e.to_string()))?
            .map(MerchantRow::into_domain)
            .ok_or(MerchantError::NotFound)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, MerchantError> {
        sqlx::query_as::<_, MerchantRow>(
            "SELECT id, name, email, api_key, is_active, created_at, updated_at
             FROM merchants WHERE email = $1"
        )
            .bind(email)
            .fetch_optional(&self.pool).await
            .map_err(|e| MerchantError::DatabaseError(e.to_string()))
            .map(|opt| opt.map(MerchantRow::into_domain))
    }

    async fn find_by_api_key(&self, api_key: &str) -> Result<Option<Merchant>, MerchantError> {
        sqlx::query_as::<_, MerchantRow>(
            "SELECT id, name, email, api_key, is_active, created_at, updated_at
             FROM merchants WHERE api_key = $1"
        )
            .bind(api_key)
            .fetch_optional(&self.pool).await
            .map_err(|e| MerchantError::DatabaseError(e.to_string()))
            .map(|opt| opt.map(MerchantRow::into_domain))
    }

    async fn find_all(&self) -> Result<Vec<Merchant>, MerchantError> {
        sqlx::query_as::<_, MerchantRow>(
            "SELECT id, name, email, api_key, is_active, created_at, updated_at
             FROM merchants ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool).await
            .map_err(|e| MerchantError::DatabaseError(e.to_string()))
            .map(|rows| rows.into_iter().map(MerchantRow::into_domain).collect())
    }

    async fn update(&self, merchant: &Merchant) -> Result<(), MerchantError> {
        sqlx
            ::query("UPDATE merchants SET name = $1, is_active = $2, updated_at = $3 WHERE id = $4")
            .bind(&merchant.name)
            .bind(merchant.is_active)
            .bind(merchant.updated_at)
            .bind(merchant.id)
            .execute(&self.pool).await
            .map_err(|e| MerchantError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct MerchantRow {
    id: Uuid,
    name: String,
    email: String,
    api_key: String,
    is_active: bool,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl MerchantRow {
    fn into_domain(self) -> Merchant {
        Merchant {
            id: self.id,
            name: self.name,
            email: self.email,
            api_key: self.api_key,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
