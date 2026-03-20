use crate::db::config_dao::{
    get_client_id, get_client_secret, get_jwt_private_key, get_state_private_key,
    get_state_public_key,
};
use crate::db::idp_dao::{dao_get_idp_by_id, dao_get_idps, Idp};
use crate::models::oauth2::{AuthorizationCodeResponse, IdpResponse};
use crate::models::service_error::ServiceError::Unauthorized;
use crate::models::tms_internal::OAuthState;
use crate::utils::jwt_utils::{JwtDecoderBuilder, JwtEncoderBuilder};
use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};

pub async fn get_idps(pool: &PgPool) -> Result<HashSet<IdpResponse>> {
    let mut tx = pool.begin().await?;
    let idps = dao_get_idps(&mut tx).await?;
    tx.commit().await?;

    let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    idps.iter().for_each(|idp| {
        idp_result.insert(idp.clone().into());
    });
    Ok(idp_result)
}

pub async fn handle_callback(
    pool: &PgPool,
    state: &String,
    code: &String,
    cookie_state: &String,
) -> Result<String> {
    if !cookie_state.eq(state) {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    }

    let decoded_state = decode_state(state)
        .await
        .context("Unable to decode state query param")?;
    dbg!(&decoded_state);

    let mut tx = pool.begin().await?;
    let idp = dao_get_idp_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;

    let token: AuthorizationCodeResponse = exchange_code_for_token(&idp, code).await?;
    dbg!(&token);

    let claims = decode_access_token(&idp, &token.id_token).await?;
    dbg!(&claims);
    make_auth_token(claims).await
}

pub async fn decode_state(state_string: &String) -> Result<OAuthState> {
    let decoding_key = DecodingKey::from_rsa_pem(&get_state_public_key().as_bytes())?;

    JwtDecoderBuilder::builder()
        .public_key(decoding_key)
        .decode::<OAuthState>(&state_string)
        .await
}

pub async fn encode_state(oauth_state: OAuthState) -> Result<String> {
    let header = Header::new(Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(get_state_private_key().as_bytes())?;

    JwtEncoderBuilder::builder(header, oauth_state, key)
        .encode()
        .await
}

pub async fn exchange_code_for_token<R>(idp: &Idp, code: &String) -> Result<R>
where
    R: for<'a> Deserialize<'a>,
{
    let form_params = [
        ("grant_type", "authorization_code".to_string()),
        ("client_id", get_client_id()),
        ("client_secret", get_client_secret()),
        ("code", code.to_owned()),
    ];
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .send()
        .await
        .context("Error getting response body")?;

    let token_string = response
        .text()
        .await
        .context("Error getting response body")?;

    serde_json::from_str::<R>(&token_string).context("Error deserializing token response body")
}

pub async fn decode_access_token<T>(idp: &Idp, id_token: &String) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let audience = HashSet::from([get_client_id()]);

    JwtDecoderBuilder::builder()
        .jwks_url(&idp.oauth2_jwks_url)
        .audience(audience)
        .decode(id_token)
        .await
        .context("Error decoding JWT")
}

pub async fn make_auth_token(claims: HashMap<String, Value>) -> Result<String> {
    let header = Header::new(Algorithm::RS256);
    let encoding_key = EncodingKey::from_rsa_pem(&get_jwt_private_key().into_bytes())?;

    // TODO: add kid, alg, and jti in header ... maybe other stuff?
    JwtEncoderBuilder::builder(header, claims, encoding_key)
        .encode()
        .await
}
