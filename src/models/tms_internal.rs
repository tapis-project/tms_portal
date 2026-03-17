use serde::{Deserialize, Serialize};
use std::result;

pub(crate) type TmsResult<T> = result::Result<T, String>;

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthState {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub idp_id: String,
    pub exp: u64,
}
