use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettlementError {
    #[error("Settlement not found")]
    NotFound,

    #[error("Settlement batch not found")]
    BatchNotFound,

    #[error("Settlement already exists for payment")]
    AlreadyExists,

    #[error("Invalid status transition")]
    InvalidStatusTransition,

    #[error("Batch is not open")]
    BatchNotOpen,

    #[error("Database error: {0}")]
    DatabaseError(String),
}