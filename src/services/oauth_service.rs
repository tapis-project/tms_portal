use crate::db::client_dao::db_get_client_by_id;
use crate::db::config_dao::db_get_state_key;
use crate::db::idp_dao::{db_get_idp_by_id, db_get_idps, Idp};
use crate::services::service_error::ServiceError::Unauthorized;
use crate::utils::jwt_utils::{JwtDecoderBuilder, JwtEncoderBuilder};
use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Clone)]
pub struct IdpResponse {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub oauth2_token_url: String,
    pub user_info_url: Option<String>,
}
impl From<Idp> for IdpResponse {
    fn from(value: Idp) -> Self {
        IdpResponse {
            id: value.id,
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthState {
    // TODO: generate crypto random nonce (or something)
    // TODO: can the expiration work better?
    pub idp_id: String,
    pub exp: u64,
}
pub async fn get_idps(pool: &PgPool) -> Result<HashSet<IdpResponse>> {
    let mut tx = pool.begin().await?;
    let idps = db_get_idps(&mut tx).await?;
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
    client_id: &String,
    cookie_state: &String,
) -> Result<String> {
    if !cookie_state.eq(state) {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    }

    let decoded_state = decode_state(pool, state)
        .await
        .context("Unable to decode state query param")?;
    dbg!(&decoded_state);

    let mut tx = pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    tx.commit().await?;

    let token: AuthorizationCodeResponse = exchange_code_for_token(&idp, code).await?;
    dbg!(&token);

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    dbg!(&claims);
    make_auth_token(pool, client_id, claims).await
}

pub async fn decode_state(pool: &PgPool, state_string: &String) -> Result<OAuthState> {
    let mut tx = pool.begin().await?;
    let keys = db_get_state_key(&mut tx).await?;
    tx.commit().await?;
    let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.public_key.as_bytes())?);

    JwtDecoderBuilder::builder()
        .public_key(&decoding_key)
        .decode::<OAuthState>(&state_string)
        .await
}

pub async fn encode_state(pool: &PgPool, oauth_state: OAuthState) -> Result<String> {
    let header = Header::new(Algorithm::RS256);
    let mut tx = pool.begin().await?;
    let keys = db_get_state_key(&mut tx).await?;
    tx.commit().await?;
    let key = EncodingKey::from_rsa_pem(&keys.private_key.as_bytes())?;

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
        //        ("client_id", get_client_id()),
        //        ("client_secret", get_client_secret()),
        (
            "redirect_uri",
            "http://localhost:8080/oauth2/callback".to_string(),
        ),
        ("code", code.to_owned()),
    ];
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .basic_auth(idp.client_id.clone(), Some(idp.client_secret.clone()))
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
    let audience = HashSet::from([idp.client_id.to_owned()]);
    let public_key = match idp.oauth2_public_key {
        Some(ref key) => Some(DecodingKey::from_rsa_pem(key.as_bytes())?),
        None => None,
    };

    JwtDecoderBuilder::builder()
        .jwks_url(&idp.oauth2_jwks_url)
        .public_key(&public_key)
        .audience(audience)
        .decode(id_token)
        .await
        .context("Error decoding JWT")
}

#[derive(Debug, Serialize, Deserialize)]
struct TmsTokenClaims {
    jti: String,
    iss: String,
    sub: String,
    #[serde(rename = "tms/token_type")]
    tms_token_type: String,
    #[serde(rename = "tms/username")]
    tms_username: String,
    #[serde(rename = "tms/client_id")]
    tms_client_id: String,
    #[serde(rename = "tms/grant_type")]
    tms_grant_type: String,
    #[serde(rename = "tms/account_type")]
    tms_account_type: String,
    exp: String,
}

pub async fn make_auth_token(
    db_pool: &PgPool,
    client_id: &String,
    claims: HashMap<String, Value>,
) -> Result<String> {
    let header = Header::new(Algorithm::RS256);
    let mut tx = db_pool.begin().await?;
    let client = db_get_client_by_id(&mut tx, &client_id).await?;
    let encoding_key = EncodingKey::from_rsa_pem(&client.jwt_private_key.into_bytes())?;
    let tms_token_clams = TmsTokenClaims {
        jti: Uuid::new_v4().to_string(),
        iss: String::from("TODO: .."),
        sub: String::from("TODO: .."),
        tms_token_type: String::from("access"),
        tms_username: String::from("TODO: .."),
        tms_client_id: String::from("TODO: .."),
        tms_grant_type: String::from("password"),
        tms_account_type: String::from("user"),
        exp: String::from("TODO: .."),
    };

    // TODO: add kid, alg, and jti in header ... maybe other stuff?
    JwtEncoderBuilder::builder(header, tms_token_clams, encoding_key)
        .encode()
        .await
}
