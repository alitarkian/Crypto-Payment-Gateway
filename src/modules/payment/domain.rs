use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Detected,
    Confirmed,
    Failed,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentStatus::Detected => "detected",
            PaymentStatus::Confirmed => "confirmed",
            PaymentStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "confirmed" => PaymentStatus::Confirmed,
            "failed" => PaymentStatus::Failed,
            _ => PaymentStatus::Detected,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Payment {
    pub id: Uuid,
    pub invoice_id: Uuid,
    pub wallet_id: Uuid,
    pub merchant_id: Uuid,
    pub signature: String,
    pub amount: Decimal,
    pub asset: String,
    pub blockchain: String,
    pub status: PaymentStatus,
    pub detected_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(
        invoice_id: Uuid,
        wallet_id: Uuid,
        merchant_id: Uuid,
        signature: String,
        amount: Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            invoice_id,
            wallet_id,
            merchant_id,
            signature,
            amount,
            asset: "USDC".to_string(),
            blockchain: "solana".to_string(),
            status: PaymentStatus::Detected,
            detected_at: now,
            confirmed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn confirm(&mut self) {
        self.status = PaymentStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}