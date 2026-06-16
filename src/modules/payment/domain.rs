use chrono::{ DateTime, Utc };
use rust_decimal::Decimal;
use uuid::Uuid;

use super::errors::PaymentError;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
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

    pub fn try_from_str(s: &str) -> Result<Self, PaymentError> {
        match s {
            "detected" => Ok(PaymentStatus::Detected),
            "confirmed" => Ok(PaymentStatus::Confirmed),
            "failed" => Ok(PaymentStatus::Failed),
            other => Err(PaymentError::InvalidStatus(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
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
        blockchain: String,
        asset: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            invoice_id,
            wallet_id,
            merchant_id,
            signature,
            amount,
            asset,
            blockchain,
            status: PaymentStatus::Detected,
            detected_at: now,
            confirmed_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Transition payment to Confirmed state.
    ///
    /// TODO(phase-13): Replace immediate confirmation with a block-confirmation
    /// oracle that waits for N confirmations before calling this.
    pub fn confirm(&mut self) {
        self.status = PaymentStatus::Confirmed;
        self.confirmed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn make_payment(blockchain: &str, asset: &str) -> Payment {
        Payment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            "sig_abc123".to_string(),
            dec!(100.00),
            blockchain.to_string(),
            asset.to_string(),
        )
    }

    // ── PaymentStatus::try_from_str ───────────────────────────────────────────

    #[test]
    fn try_from_str_detected() {
        assert_eq!(PaymentStatus::try_from_str("detected").unwrap(), PaymentStatus::Detected);
    }

    #[test]
    fn try_from_str_confirmed() {
        assert_eq!(PaymentStatus::try_from_str("confirmed").unwrap(), PaymentStatus::Confirmed);
    }

    #[test]
    fn try_from_str_failed() {
        assert_eq!(PaymentStatus::try_from_str("failed").unwrap(), PaymentStatus::Failed);
    }

    #[test]
    fn try_from_str_invalid_returns_err() {
        let result = PaymentStatus::try_from_str("unknown_garbage");
        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::InvalidStatus(s) => assert_eq!(s, "unknown_garbage"),
            other => panic!("Expected InvalidStatus, got {:?}", other),
        }
    }

    #[test]
    fn try_from_str_empty_returns_err() {
        assert!(PaymentStatus::try_from_str("").is_err());
    }

    // ── Payment::new() ────────────────────────────────────────────────────────

    #[test]
    fn new_payment_stores_blockchain_and_asset() {
        let p = make_payment("ethereum", "USDC");
        assert_eq!(p.blockchain, "ethereum");
        assert_eq!(p.asset, "USDC");
    }

    #[test]
    fn new_payment_initial_status_is_detected() {
        let p = make_payment("solana", "USDC");
        assert_eq!(p.status, PaymentStatus::Detected);
    }

    #[test]
    fn new_payment_confirmed_at_is_none() {
        let p = make_payment("solana", "USDC");
        assert!(p.confirmed_at.is_none());
    }

    // ── Payment::confirm() ────────────────────────────────────────────────────

    #[test]
    fn confirm_transitions_to_confirmed() {
        let mut p = make_payment("solana", "USDC");
        p.confirm();
        assert_eq!(p.status, PaymentStatus::Confirmed);
    }

    #[test]
    fn confirm_sets_confirmed_at() {
        let mut p = make_payment("solana", "USDC");
        assert!(p.confirmed_at.is_none());
        p.confirm();
        assert!(p.confirmed_at.is_some());
    }

    #[test]
    fn confirm_updates_updated_at() {
        let mut p = make_payment("solana", "USDC");
        let before = p.updated_at;
        // Sleep 1ms so timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(2));
        p.confirm();
        assert!(p.updated_at >= before);
    }
}
