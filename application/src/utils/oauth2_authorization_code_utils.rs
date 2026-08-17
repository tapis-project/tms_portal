use std::collections::HashSet;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::debug;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use crate::obj_model;
use crate::obj_model::identity_provider::IdentityProvider;

pub const ROOT_COOKIE_PATH: &str = "/";
pub const CLIENT_ID_TMS: &str = "tms";
pub const TOKEN_COOKIE_NAME: &str = "tmstoken";
pub const STATE_COOKIE_NAME: &str = "state_cookie";

#[derive(Debug, Deserialize)]
pub struct AuthCodeQueryParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct ListResourceProviderRequestParams {
    pub linked_only: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2State {
    // TODO: can the expiration work better?
    pub tms_identity: String,           // the tms "cloud identity"
    pub idp_id: String,                 // id of the cloud identity provider a.k.a. login identity provider
    pub client_id: String,              // client id of the tms client
    pub exp: u64,                       // TODO: this is supposed to be an expiration for the state, but it should
                                        // TODO: have a date time or something maybe?  This needs work.
    pub redirect_uri: String,           // redirect_uri requested by the tms client
    pub client_state: Option<String>,   // state provided to authorize endpoint by tms client
    pub nonce: u32,                     // nonce - used to help prevent replay attacks
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

pub async fn decode_access_token<T>(
    idp: &obj_model::identity_provider::IdentityProvider,
    id_token: &String,
) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let audience = HashSet::from([idp.client_id.to_owned()]);
    let mut builder = JwtDecoderBuilder::builder().jwks_url(&idp.oauth2_jwks_url);
    if let Some(key) = &idp.oauth2_public_key {
        builder = builder.public_key(&key.as_bytes());
    }
    builder
        .audience(audience)
        .decode(id_token)
        .await
        .context("Error decoding JWT")
}