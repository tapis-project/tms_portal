use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Method Not allowed: {0}")]
    MethodNotAllowed(String),
}
