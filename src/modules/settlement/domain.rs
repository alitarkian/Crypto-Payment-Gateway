use chrono::{ DateTime, NaiveDate, Utc };
use rust_decimal::Decimal;
use uuid::Uuid;

pub const PLATFORM_FEE_RATE: Decimal = Decimal::from_parts(100, 0, 0, false, 4); // 0.0100 = 1%

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum SettlementStatus {
    Pending,
    Processing,
    ReadyToPay,
    Paid,
    Failed,
}

impl SettlementStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SettlementStatus::Pending => "pending",
            SettlementStatus::Processing => "processing",
            SettlementStatus::ReadyToPay => "ready_to_pay",
            SettlementStatus::Paid => "paid",
            SettlementStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "processing" => SettlementStatus::Processing,
            "ready_to_pay" => SettlementStatus::ReadyToPay,
            "paid" => SettlementStatus::Paid,
            "failed" => SettlementStatus::Failed,
            _ => SettlementStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum BatchStatus {
    Open,
    Closed,
    Processing,
    Completed,
    Failed,
}

impl BatchStatus {
    pub fn as_str(&self) -> &str {
        match self {
            BatchStatus::Open => "open",
            BatchStatus::Closed => "closed",
            BatchStatus::Processing => "processing",
            BatchStatus::Completed => "completed",
            BatchStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "closed" => BatchStatus::Closed,
            "processing" => BatchStatus::Processing,
            "completed" => BatchStatus::Completed,
            "failed" => BatchStatus::Failed,
            _ => BatchStatus::Open,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Settlement {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub invoice_id: Uuid,
    pub payment_id: Uuid,
    pub gross_amount: Decimal,
    pub fee_rate: Decimal,
    pub fee_amount: Decimal,
    pub net_amount: Decimal,
    pub asset: String,
    pub blockchain: String,
    pub status: SettlementStatus,
    pub batch_id: Option<Uuid>,
    pub settled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Settlement {
    pub fn new(
        merchant_id: Uuid,
        invoice_id: Uuid,
        payment_id: Uuid,
        gross_amount: Decimal,
        fee_rate: Decimal
    ) -> Self {
        let fee_amount = (gross_amount * fee_rate).round_dp(8);
        let net_amount = gross_amount - fee_amount;
        let now = Utc::now();

        Self {
            id: Uuid::new_v4(),
            merchant_id,
            invoice_id,
            payment_id,
            gross_amount,
            fee_rate,
            fee_amount,
            net_amount,
            asset: "USDC".to_string(),
            blockchain: "solana".to_string(),
            status: SettlementStatus::Pending,
            batch_id: None,
            settled_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn assign_to_batch(&mut self, batch_id: Uuid) {
        self.batch_id = Some(batch_id);
        self.status = SettlementStatus::Processing;
        self.updated_at = Utc::now();
    }

    pub fn mark_ready(&mut self) {
        self.status = SettlementStatus::ReadyToPay;
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn mark_paid(&mut self) {
        self.status = SettlementStatus::Paid;
        self.settled_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn mark_failed(&mut self) {
        self.status = SettlementStatus::Failed;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SettlementBatch {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub status: BatchStatus,
    pub total_gross: Decimal,
    pub total_fee: Decimal,
    pub total_net: Decimal,
    pub settlement_count: i32,
    pub asset: String,
    pub blockchain: String,
    pub period_date: NaiveDate,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SettlementBatch {
    pub fn new(merchant_id: Uuid, period_date: NaiveDate) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            merchant_id,
            status: BatchStatus::Open,
            total_gross: Decimal::ZERO,
            total_fee: Decimal::ZERO,
            total_net: Decimal::ZERO,
            settlement_count: 0,
            asset: "USDC".to_string(),
            blockchain: "solana".to_string(),
            period_date,
            completed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_settlement(&mut self, settlement: &Settlement) {
        self.total_gross += settlement.gross_amount;
        self.total_fee += settlement.fee_amount;
        self.total_net += settlement.net_amount;
        self.settlement_count += 1;
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn close(&mut self) {
        self.status = BatchStatus::Closed;
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn mark_completed(&mut self) {
        self.status = BatchStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    #[allow(dead_code)]
    pub fn mark_failed(&mut self) {
        self.status = BatchStatus::Failed;
        self.updated_at = Utc::now();
    }
}
