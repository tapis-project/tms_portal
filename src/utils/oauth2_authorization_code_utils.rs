use crate::db::identity_provider_dao::IdentityProvider;
use crate::db::keys_dao::db_get_key_by_id;
use crate::services::login_service::TmsTokenClaims;
use crate::services::service_error::ServiceError::BadRequest;
use crate::utils::jwt_utils::JwtDecoderBuilder;
use crate::utils::oauth2_authorization_code_utils;
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
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
    pub idp_id: String,
    pub client_id: String,
    pub exp: u64,
    pub redirect_uri: String,
}

pub async fn get_token_claims(db_pool: &PgPool, token: &String) -> Result<TmsTokenClaims> {
    let token_header = decode_header(token)?;
    let mut tx = db_pool.begin().await?;
    let key = match token_header.kid {
        Some(kid) => db_get_key_by_id(&mut tx, &kid).await,
        None => return Err(BadRequest(String::from("Unable to find key for jwt")).into()),
    }?;
    tx.commit().await?;

    let tms_token_claims: TmsTokenClaims = JwtDecoderBuilder::builder()
        .public_key(key.jwt_public_key.as_bytes())
        .decode(token)
        .await?;

    Ok(tms_token_claims)
}

pub async fn get_token_for_provider<R>(
    idp: &IdentityProvider,
    callback_url: &String,
    code: &String,
) -> Result<R>
where
    R: DeserializeOwned,
{
    oauth2_authorization_code_utils::exchange_code_for_token(
        &idp.oauth2_token_url,
        &idp.client_id,
        &idp.client_secret,
        callback_url,
        code,
    )
    .await
}

pub async fn exchange_code_for_token<R>(
    oauth2_token_url: &String,
    provider_client_id: &String,
    provider_client_secret: &String,
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
        .post(oauth2_token_url)
        .form(&form_params)
        .basic_auth(
            provider_client_id.clone(),
            Some(provider_client_secret.clone()),
        )
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
