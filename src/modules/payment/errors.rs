use thiserror::Error;

#[derive(Debug, Error)]
pub enum PaymentError {
    #[allow(dead_code)]
    #[error("Payment not found")]
    NotFound,

    #[error("Invoice not found")]
    InvoiceNotFound,

    #[error("Invoice is not payable")]
    InvoiceNotPayable,

    #[error("Duplicate transaction signature")]
    DuplicateSignature,

    #[error("Amount mismatch: expected {expected}, got {received}")] AmountMismatch {
        expected: String,
        received: String,
    },

    #[error("Database error: {0}")] DatabaseError(String),
}
