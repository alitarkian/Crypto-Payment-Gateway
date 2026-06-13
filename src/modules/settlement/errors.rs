use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettlementError {
    #[allow(dead_code)]
    #[error("Settlement not found")]
    NotFound,

    #[allow(dead_code)]
    #[error("Settlement batch not found")]
    BatchNotFound,

    #[error("Settlement already exists for payment")]
    AlreadyExists,

    #[allow(dead_code)]
    #[error("Invalid status transition")]
    InvalidStatusTransition,

    #[allow(dead_code)]
    #[error("Batch is not open")]
    BatchNotOpen,

    #[error("Database error: {0}")] DatabaseError(String),
}
