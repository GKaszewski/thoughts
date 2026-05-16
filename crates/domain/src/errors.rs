use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum DomainError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unique violation on field: {field}")]
    UniqueViolation { field: &'static str },
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("external service error: {0}")]
    ExternalService(String),
    #[error("internal error: {0}")]
    Internal(String),
}
