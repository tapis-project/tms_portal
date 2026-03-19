use serde::{Deserialize, Serialize};
use std::result;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TmsServiceError {
    InternalError(String),
    DatabaseError(String),
    BadRequest(String),
    Unauthorized(String),
    NotFoundError(String),
}
pub(crate) type TmsResult<T> = result::Result<T, TmsServiceError>;
#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthState {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub idp_id: String,
    pub exp: u64,
}
