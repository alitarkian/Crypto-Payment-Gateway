use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::modules::payment::{
    domain::{Payment, PaymentStatus},
    errors::PaymentError,
    repository::PaymentRepository,
};

pub struct PostgresPaymentRepository {
    pool: PgPool,
}

impl PostgresPaymentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PaymentRow {
    id: Uuid,
    invoice_id: Uuid,
    wallet_id: Uuid,
    merchant_id: Uuid,
    signature: String,
    amount: Decimal,
    asset: String,
    blockchain: String,
    status: String,
    detected_at: DateTime<Utc>,
    confirmed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PaymentRow {
    fn into_domain(self) -> Result<Payment, PaymentError> {
        let status = PaymentStatus::try_from_str(&self.status)?;
        Ok(Payment {
            id: self.id,
            invoice_id: self.invoice_id,
            wallet_id: self.wallet_id,
            merchant_id: self.merchant_id,
            signature: self.signature,
            amount: self.amount,
            asset: self.asset,
            blockchain: self.blockchain,
            status,
            detected_at: self.detected_at,
            confirmed_at: self.confirmed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[async_trait]
impl PaymentRepository for PostgresPaymentRepository {
    async fn save(&self, payment: &Payment) -> Result<(), PaymentError> {
        sqlx::query(
            r#"
            INSERT INTO payments (id, invoice_id, wallet_id, merchant_id, signature, amount, asset, blockchain, status, detected_at, confirmed_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::text::payment_status, $10, $11, $12, $13)
            "#,
        )
        .bind(payment.id)
        .bind(payment.invoice_id)
        .bind(payment.wallet_id)
        .bind(payment.merchant_id)
        .bind(&payment.signature)
        .bind(payment.amount)
        .bind(&payment.asset)
        .bind(&payment.blockchain)
        .bind(payment.status.as_str())
        .bind(payment.detected_at)
        .bind(payment.confirmed_at)
        .bind(payment.created_at)
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(db_err)
                if db_err.constraint() == Some("payments_signature_key") =>
            {
                PaymentError::DuplicateSignature
            }
            _ => PaymentError::DatabaseError(e.to_string()),
        })?;

        Ok(())
    }

    async fn update(&self, payment: &Payment) -> Result<(), PaymentError> {
        sqlx::query(
            r#"
            UPDATE payments
            SET status      = $2::text::payment_status,
                confirmed_at = $3,
                updated_at   = $4
            WHERE id = $1
            "#,
        )
        .bind(payment.id)
        .bind(payment.status.as_str())
        .bind(payment.confirmed_at)
        .bind(payment.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Payment, PaymentError> {
        sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, invoice_id, wallet_id, merchant_id, signature, amount, asset, blockchain,
                      status::TEXT as status, detected_at, confirmed_at, created_at, updated_at
               FROM payments WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?
        .ok_or(PaymentError::NotFound)?
        .into_domain()
    }

    async fn find_by_signature(&self, signature: &str) -> Result<Option<Payment>, PaymentError> {
        let row = sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, invoice_id, wallet_id, merchant_id, signature, amount, asset, blockchain,
                      status::TEXT as status, detected_at, confirmed_at, created_at, updated_at
               FROM payments WHERE signature = $1"#,
        )
        .bind(signature)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;

        row.map(PaymentRow::into_domain).transpose()
    }

    async fn find_by_invoice_id(&self, invoice_id: Uuid) -> Result<Vec<Payment>, PaymentError> {
        let rows = sqlx::query_as::<_, PaymentRow>(
            r#"SELECT id, invoice_id, wallet_id, merchant_id, signature, amount, asset, blockchain,
                      status::TEXT as status, detected_at, confirmed_at, created_at, updated_at
               FROM payments WHERE invoice_id = $1
               ORDER BY created_at DESC"#,
        )
        .bind(invoice_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;

        rows.into_iter().map(PaymentRow::into_domain).collect()
    }

    async fn find_signatures_by_blockchain(
        &self,
        blockchain: &str,
    ) -> Result<Vec<String>, PaymentError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT signature FROM payments WHERE blockchain = $1"#,
        )
        .bind(blockchain)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PaymentError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|(s,)| s).collect())
    }
}