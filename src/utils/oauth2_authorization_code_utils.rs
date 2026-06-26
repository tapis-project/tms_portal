use crate::db::identity_provider_dao::IdentityProvider;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub const ROOT_COOKIE_PATH: &str = "/";
pub const CLIENT_ID_TMS: &str = "tms";
pub const TOKEN_COOKIE_NAME: &str = "tmstoken";
pub const STATE_COOKIE_NAME: &str = "state_cookie";

#[derive(Debug, Deserialize)]
pub struct AuthCodeQueryParams {
    pub code: String,
    pub state: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2State {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub tms_identity: String,
    pub idp_id: String,
    pub client_id: String,
    pub exp: u64,
    pub redirect_uri: String,
}

/*
Exchanges an auth code for an auth token.  The parameter <R> is the structure
that it is deserialized into.
 */
pub async fn get_token_for_provider<R>(
    idp: &IdentityProvider,
    callback_url: &String,
    code: &String,
) -> Result<R>
where
    R: DeserializeOwned,
{
    debug!("exchange_code_for_token called");
    let form_params = [
        ("grant_type", &"authorization_code".to_string()),
        ("redirect_uri", callback_url),
        ("code", &code.to_owned()),
    ];
    debug!("Form params: {:?}", form_params);
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .basic_auth(idp.client_id.clone(), Some(idp.client_secret.clone()))
        .send()
        .await
        .context("Error getting response body")?;
    debug!("Response from exchange code: {:?}", response);

    let token_string = response
        .text()
        .await
        .context("Error getting response body")?;

    serde_json::from_str::<R>(&token_string).context("Error deserializing token response body")
}
