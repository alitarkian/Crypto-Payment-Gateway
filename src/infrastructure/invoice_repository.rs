use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{ FromRow, PgPool };
use uuid::Uuid;
use chrono::{ DateTime, Utc };

use crate::modules::invoice::{
    domain::{ Invoice, InvoiceStatus },
    errors::InvoiceError,
    repository::InvoiceRepository,
};

pub struct PostgresInvoiceRepository {
    pool: PgPool,
}

impl PostgresInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct InvoiceRow {
    id: Uuid,
    merchant_id: Uuid,
    wallet_id: Uuid,
    amount: Decimal,
    asset: String,
    blockchain: String,
    status: String,
    description: Option<String>,
    expires_at: DateTime<Utc>,
    paid_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl InvoiceRow {
    fn into_domain(self) -> Result<Invoice, InvoiceError> {
        Ok(Invoice {
            id: self.id,
            merchant_id: self.merchant_id,
            wallet_id: self.wallet_id,
            amount: self.amount,
            asset: self.asset,
            blockchain: self.blockchain,
            status: InvoiceStatus::from_str(&self.status)?,
            description: self.description,
            expires_at: self.expires_at,
            paid_at: self.paid_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl InvoiceRepository for PostgresInvoiceRepository {
    async fn save(&self, invoice: &Invoice) -> Result<(), InvoiceError> {
        sqlx
            ::query(
                r#"
            INSERT INTO invoices (id, merchant_id, wallet_id, amount, asset, blockchain, status, description, expires_at, paid_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7::text::invoice_status, $8, $9, $10, $11, $12)
            "#
            )
            .bind(invoice.id)
            .bind(invoice.merchant_id)
            .bind(invoice.wallet_id)
            .bind(invoice.amount)
            .bind(&invoice.asset)
            .bind(&invoice.blockchain)
            .bind(invoice.status.as_str())
            .bind(&invoice.description)
            .bind(invoice.expires_at)
            .bind(invoice.paid_at)
            .bind(invoice.created_at)
            .bind(invoice.updated_at)
            .execute(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update(&self, invoice: &Invoice) -> Result<(), InvoiceError> {
        sqlx
            ::query(
                r#"
            UPDATE invoices
            SET status = $2::text::invoice_status, paid_at = $3, updated_at = $4
            WHERE id = $1
            "#
            )
            .bind(invoice.id)
            .bind(invoice.status.as_str())
            .bind(invoice.paid_at)
            .bind(invoice.updated_at)
            .execute(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Invoice, InvoiceError> {
        sqlx::query_as::<_, InvoiceRow>(
            r#"SELECT id, merchant_id, wallet_id, amount, asset, blockchain,
                      status::TEXT as status, description, expires_at, paid_at, created_at, updated_at
               FROM invoices WHERE id = $1"#
        )
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?
            .ok_or(InvoiceError::NotFound)?
            .into_domain()
    }

    async fn find_by_merchant_id(&self, merchant_id: Uuid) -> Result<Vec<Invoice>, InvoiceError> {
        sqlx::query_as::<_, InvoiceRow>(
            r#"SELECT id, merchant_id, wallet_id, amount, asset, blockchain,
                      status::TEXT as status, description, expires_at, paid_at, created_at, updated_at
               FROM invoices WHERE merchant_id = $1
               ORDER BY created_at DESC"#
        )
            .bind(merchant_id)
            .fetch_all(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(InvoiceRow::into_domain)
            .collect()
    }

    async fn find_pending_expired(&self) -> Result<Vec<Invoice>, InvoiceError> {
        sqlx::query_as::<_, InvoiceRow>(
            r#"SELECT id, merchant_id, wallet_id, amount, asset, blockchain,
                      status::TEXT as status, description, expires_at, paid_at, created_at, updated_at
               FROM invoices
               WHERE status = 'pending'::invoice_status AND expires_at < NOW()"#
        )
            .fetch_all(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(InvoiceRow::into_domain)
            .collect()
    }

    async fn find_pending_active(&self) -> Result<Vec<Invoice>, InvoiceError> {
        sqlx::query_as::<_, InvoiceRow>(
            r#"SELECT id, merchant_id, wallet_id, amount, asset, blockchain,
                      status::TEXT as status, description, expires_at, paid_at, created_at, updated_at
               FROM invoices
               WHERE status = 'pending'::invoice_status AND expires_at > NOW()"#
        )
            .fetch_all(&self.pool).await
            .map_err(|e| InvoiceError::DatabaseError(e.to_string()))?
            .into_iter()
            .map(InvoiceRow::into_domain)
            .collect()
    }
}
