use async_trait::async_trait;
use chrono::{ DateTime, NaiveDate, Utc };
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::modules::settlement::{
    domain::{ BatchStatus, Settlement, SettlementBatch, SettlementStatus },
    errors::SettlementError,
    repository::SettlementRepository,
};

pub struct PostgresSettlementRepository {
    pool: PgPool,
}

impl PostgresSettlementRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct SettlementRow {
    id: Uuid,
    merchant_id: Uuid,
    invoice_id: Uuid,
    payment_id: Uuid,
    gross_amount: Decimal,
    fee_rate: Decimal,
    fee_amount: Decimal,
    net_amount: Decimal,
    asset: String,
    blockchain: String,
    status: String,
    batch_id: Option<Uuid>,
    settled_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SettlementRow {
    fn into_domain(self) -> Settlement {
        Settlement {
            id: self.id,
            merchant_id: self.merchant_id,
            invoice_id: self.invoice_id,
            payment_id: self.payment_id,
            gross_amount: self.gross_amount,
            fee_rate: self.fee_rate,
            fee_amount: self.fee_amount,
            net_amount: self.net_amount,
            asset: self.asset,
            blockchain: self.blockchain,
            status: SettlementStatus::from_str(&self.status),
            batch_id: self.batch_id,
            settled_at: self.settled_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct BatchRow {
    id: Uuid,
    merchant_id: Uuid,
    status: String,
    total_gross: Decimal,
    total_fee: Decimal,
    total_net: Decimal,
    settlement_count: i32,
    asset: String,
    blockchain: String,
    period_date: NaiveDate,
    completed_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl BatchRow {
    fn into_domain(self) -> SettlementBatch {
        SettlementBatch {
            id: self.id,
            merchant_id: self.merchant_id,
            status: BatchStatus::from_str(&self.status),
            total_gross: self.total_gross,
            total_fee: self.total_fee,
            total_net: self.total_net,
            settlement_count: self.settlement_count,
            asset: self.asset,
            blockchain: self.blockchain,
            period_date: self.period_date,
            completed_at: self.completed_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[async_trait]
impl SettlementRepository for PostgresSettlementRepository {
    async fn save_settlement(&self, s: &Settlement) -> Result<(), SettlementError> {
        sqlx
            ::query(
                r#"
            INSERT INTO settlements
                (id, merchant_id, invoice_id, payment_id, gross_amount, fee_rate, fee_amount,
                 net_amount, asset, blockchain, status, batch_id, settled_at, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::text::settlement_status,$12,$13,$14,$15)
            "#
            )
            .bind(s.id)
            .bind(s.merchant_id)
            .bind(s.invoice_id)
            .bind(s.payment_id)
            .bind(s.gross_amount)
            .bind(s.fee_rate)
            .bind(s.fee_amount)
            .bind(s.net_amount)
            .bind(&s.asset)
            .bind(&s.blockchain)
            .bind(s.status.as_str())
            .bind(s.batch_id)
            .bind(s.settled_at)
            .bind(s.created_at)
            .bind(s.updated_at)
            .execute(&self.pool).await
            .map_err(|e| {
                match e {
                    sqlx::Error::Database(ref db) if
                        db.constraint() == Some("idx_settlements_payment_id")
                    => SettlementError::AlreadyExists,
                    _ => SettlementError::DatabaseError(e.to_string()),
                }
            })?;
        Ok(())
    }

    async fn update_settlement(&self, s: &Settlement) -> Result<(), SettlementError> {
        sqlx
            ::query(
                r#"
            UPDATE settlements
            SET status = $1::text::settlement_status, batch_id = $2,
                settled_at = $3, updated_at = $4
            WHERE id = $5
            "#
            )
            .bind(s.status.as_str())
            .bind(s.batch_id)
            .bind(s.settled_at)
            .bind(s.updated_at)
            .bind(s.id)
            .execute(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_settlement_by_id(&self, id: Uuid) -> Result<Settlement, SettlementError> {
        sqlx::query_as::<_, SettlementRow>(
            r#"SELECT id, merchant_id, invoice_id, payment_id, gross_amount, fee_rate, fee_amount,
                      net_amount, asset, blockchain, status::TEXT, batch_id, settled_at,
                      created_at, updated_at
               FROM settlements WHERE id = $1"#
        )
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))?
            .ok_or(SettlementError::NotFound)
            .map(SettlementRow::into_domain)
    }

    async fn find_settlement_by_payment_id(
        &self,
        payment_id: Uuid
    ) -> Result<Option<Settlement>, SettlementError> {
        sqlx::query_as::<_, SettlementRow>(
            r#"SELECT id, merchant_id, invoice_id, payment_id, gross_amount, fee_rate, fee_amount,
                      net_amount, asset, blockchain, status::TEXT, batch_id, settled_at,
                      created_at, updated_at
               FROM settlements WHERE payment_id = $1"#
        )
            .bind(payment_id)
            .fetch_optional(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))
            .map(|opt| opt.map(SettlementRow::into_domain))
    }

    async fn find_pending_settlements(&self) -> Result<Vec<Settlement>, SettlementError> {
        sqlx::query_as::<_, SettlementRow>(
            r#"SELECT id, merchant_id, invoice_id, payment_id, gross_amount, fee_rate, fee_amount,
                      net_amount, asset, blockchain, status::TEXT, batch_id, settled_at,
                      created_at, updated_at
               FROM settlements WHERE status = 'pending'
               ORDER BY created_at ASC LIMIT 500"#
        )
            .fetch_all(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))
            .map(|rows| rows.into_iter().map(SettlementRow::into_domain).collect())
    }

    async fn find_settlements_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<Settlement>, SettlementError> {
        sqlx::query_as::<_, SettlementRow>(
            r#"SELECT id, merchant_id, invoice_id, payment_id, gross_amount, fee_rate, fee_amount,
                      net_amount, asset, blockchain, status::TEXT, batch_id, settled_at,
                      created_at, updated_at
               FROM settlements WHERE merchant_id = $1
               ORDER BY created_at DESC"#
        )
            .bind(merchant_id)
            .fetch_all(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))
            .map(|rows| rows.into_iter().map(SettlementRow::into_domain).collect())
    }

    async fn save_batch(&self, b: &SettlementBatch) -> Result<(), SettlementError> {
        sqlx
            ::query(
                r#"
            INSERT INTO settlement_batches
                (id, merchant_id, status, total_gross, total_fee, total_net,
                 settlement_count, asset, blockchain, period_date, completed_at, created_at, updated_at)
            VALUES ($1,$2,$3::text::batch_status,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
            "#
            )
            .bind(b.id)
            .bind(b.merchant_id)
            .bind(b.status.as_str())
            .bind(b.total_gross)
            .bind(b.total_fee)
            .bind(b.total_net)
            .bind(b.settlement_count)
            .bind(&b.asset)
            .bind(&b.blockchain)
            .bind(b.period_date)
            .bind(b.completed_at)
            .bind(b.created_at)
            .bind(b.updated_at)
            .execute(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn update_batch(&self, b: &SettlementBatch) -> Result<(), SettlementError> {
        sqlx
            ::query(
                r#"
            UPDATE settlement_batches
            SET status = $1::text::batch_status, total_gross = $2, total_fee = $3,
                total_net = $4, settlement_count = $5, completed_at = $6, updated_at = $7
            WHERE id = $8
            "#
            )
            .bind(b.status.as_str())
            .bind(b.total_gross)
            .bind(b.total_fee)
            .bind(b.total_net)
            .bind(b.settlement_count)
            .bind(b.completed_at)
            .bind(b.updated_at)
            .bind(b.id)
            .execute(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn find_batch_by_id(&self, id: Uuid) -> Result<SettlementBatch, SettlementError> {
        sqlx::query_as::<_, BatchRow>(
            r#"SELECT id, merchant_id, status::TEXT, total_gross, total_fee, total_net,
                      settlement_count, asset, blockchain, period_date, completed_at,
                      created_at, updated_at
               FROM settlement_batches WHERE id = $1"#
        )
            .bind(id)
            .fetch_optional(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))?
            .ok_or(SettlementError::BatchNotFound)
            .map(BatchRow::into_domain)
    }

    async fn find_open_batch_for_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Option<SettlementBatch>, SettlementError> {
        sqlx::query_as::<_, BatchRow>(
            r#"SELECT id, merchant_id, status::TEXT, total_gross, total_fee, total_net,
                      settlement_count, asset, blockchain, period_date, completed_at,
                      created_at, updated_at
               FROM settlement_batches
               WHERE merchant_id = $1 AND status = 'open'
               ORDER BY created_at DESC LIMIT 1"#
        )
            .bind(merchant_id)
            .fetch_optional(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))
            .map(|opt| opt.map(BatchRow::into_domain))
    }

    async fn find_batches_by_merchant(
        &self,
        merchant_id: Uuid
    ) -> Result<Vec<SettlementBatch>, SettlementError> {
        sqlx::query_as::<_, BatchRow>(
            r#"SELECT id, merchant_id, status::TEXT, total_gross, total_fee, total_net,
                      settlement_count, asset, blockchain, period_date, completed_at,
                      created_at, updated_at
               FROM settlement_batches WHERE merchant_id = $1
               ORDER BY created_at DESC"#
        )
            .bind(merchant_id)
            .fetch_all(&self.pool).await
            .map_err(|e| SettlementError::DatabaseError(e.to_string()))
            .map(|rows| rows.into_iter().map(BatchRow::into_domain).collect())
    }
}
