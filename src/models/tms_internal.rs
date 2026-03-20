use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TmsServiceError {
    InternalError(String),
    DatabaseError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFoundError(String),
}
