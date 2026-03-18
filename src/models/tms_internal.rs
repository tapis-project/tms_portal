use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result;

pub(crate) type TmsResult<T> = result::Result<T, String>;
pub(crate) type TmsResultNew<T> = Result<T, Box<dyn Error>>;
#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthState {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub idp_id: String,
    pub exp: u64,
}
