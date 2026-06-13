use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;
use super::errors::InvoiceError;

#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Pending,
    Paid,
    Expired,
    Cancelled,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Expired => "expired",
            InvoiceStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, InvoiceError> {
        match s {
            "draft" => Ok(InvoiceStatus::Draft),
            "pending" => Ok(InvoiceStatus::Pending),
            "paid" => Ok(InvoiceStatus::Paid),
            "expired" => Ok(InvoiceStatus::Expired),
            "cancelled" => Ok(InvoiceStatus::Cancelled),
            _ => Err(InvoiceError::NotFound),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Invoice {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub wallet_id: Uuid,
    pub amount: Decimal,
    pub asset: String,
    pub blockchain: String,
    pub status: InvoiceStatus,
    pub description: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    pub fn new(
        merchant_id: Uuid,
        wallet_id: Uuid,
        amount: Decimal,
        description: Option<String>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, InvoiceError> {
        if amount <= Decimal::ZERO {
            return Err(InvoiceError::InvalidAmount);
        }

        if expires_at <= Utc::now() {
            return Err(InvoiceError::InvalidExpiration);
        }

        Ok(Self {
            id: Uuid::new_v4(),
            merchant_id,
            wallet_id,
            amount,
            asset: "USDC".to_string(),
            blockchain: "solana".to_string(),
            status: InvoiceStatus::Pending,
            description,
            expires_at,
            paid_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    pub fn is_payable(&self) -> bool {
        self.status == InvoiceStatus::Pending && self.expires_at > Utc::now()
    }

    pub fn mark_paid(&mut self) -> Result<(), InvoiceError> {
        match self.status {
            InvoiceStatus::Paid => Err(InvoiceError::AlreadyPaid),
            InvoiceStatus::Expired => Err(InvoiceError::AlreadyExpired),
            InvoiceStatus::Cancelled => Err(InvoiceError::AlreadyCancelled),
            _ => {
                self.status = InvoiceStatus::Paid;
                self.paid_at = Some(Utc::now());
                self.updated_at = Utc::now();
                Ok(())
            }
        }
    }

    pub fn mark_expired(&mut self) -> Result<(), InvoiceError> {
        match self.status {
            InvoiceStatus::Paid => Err(InvoiceError::AlreadyPaid),
            InvoiceStatus::Expired => Err(InvoiceError::AlreadyExpired),
            InvoiceStatus::Cancelled => Err(InvoiceError::AlreadyCancelled),
            _ => {
                self.status = InvoiceStatus::Expired;
                self.updated_at = Utc::now();
                Ok(())
            }
        }
    }
}