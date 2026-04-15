use crate::db::client_dao::db_get_client_by_id;
use crate::db::config_dao::{db_get_http_config, db_get_jwt_config, db_get_state_key_id};
use crate::db::idp_dao::{db_get_idp_by_id, db_get_idps, Idp};
use crate::db::keys_dao::db_get_key_by_id;
use crate::services::service_error::ServiceError::Unauthorized;
use crate::utils::jwt_utils::{JwtDecoderBuilder, JwtEncoderBuilder};
use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::debug;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use uuid::Uuid;

const DEFAULT_ALGORITHM: &str = "RS256";

const CLAIM_SUB: &str = "sub";
const CLAIM_IDP: &str = "identity_provider";
const CLAIM_NAME: &str = "name";
const CLAIM_IDP_DISPLAY_NAME: &str = "identity_provider_display_name";
const CLAIM_ORGANIZATION: &str = "organization";
const CLAIM_BOGUS: &str = "bogus";

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
    pub client_id: String,
    pub exp: u64,
    pub redirect_uri: String,
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

    let token: AuthorizationCodeResponse = exchange_code_for_token(&pool, &idp, code).await?;
    dbg!(&token);

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    dbg!(&claims);
    make_auth_token(pool, &decoded_state.client_id, claims).await
}

pub async fn decode_state(pool: &PgPool, state_string: &String) -> Result<OAuthState> {
    let mut tx = pool.begin().await?;
    let state_key = db_get_state_key_id(&mut tx).await?;
    let keys = db_get_key_by_id(&mut tx, &state_key.kid).await?;
    tx.commit().await?;
    // let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.jwt_public_key.as_bytes())?);

    JwtDecoderBuilder::builder()
        .public_key(&keys.jwt_public_key.as_bytes())
        .decode::<OAuthState>(&state_string)
        .await
}

pub async fn encode_state(pool: &PgPool, oauth_state: OAuthState) -> Result<String> {
    let mut tx = pool.begin().await?;
    let state_key = db_get_state_key_id(&mut tx).await?;
    let keys = db_get_key_by_id(&mut tx, &state_key.kid).await?;
    tx.commit().await?;

    JwtEncoderBuilder::builder(
        oauth_state,
        &keys.jwt_private_key.as_bytes(),
        DEFAULT_ALGORITHM,
        keys.kid.as_str(),
    )
    .encode()
    .await
}

pub async fn exchange_code_for_token<R>(pool: &PgPool, idp: &Idp, code: &String) -> Result<R>
where
    R: for<'a> Deserialize<'a>,
{
    debug!("exchange_code_for_token called");
    let mut tx = pool.begin().await?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;
    let callback_url = &http_config.get_callback_url();
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

pub async fn decode_access_token<T>(idp: &Idp, id_token: &String) -> Result<T>
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

#[derive(Debug, Serialize, Deserialize)]
struct TmsTokenClaims {
    jti: Value,
    iss: Value,
    sub: Value,
    #[serde(rename = "tms/token_type")]
    tms_token_type: Value,
    #[serde(rename = "tms/username")]
    tms_username: Value,
    #[serde(rename = "tms/client_id")]
    tms_client_id: Value,
    #[serde(rename = "tms/grant_type")]
    tms_grant_type: Value,
    #[serde(rename = "tms/account_type")]
    tms_account_type: Value,
    #[serde(rename = "tms/name", skip_serializing_if = "Option::is_none")]
    tms_name: Option<Value>,
    #[serde(
        rename = "tms/identity_provider_display_name",
        skip_serializing_if = "Option::is_none"
    )]
    tms_idp_display_name: Option<Value>,
    #[serde(rename = "tms/organization", skip_serializing_if = "Option::is_none")]
    tms_organization: Option<Value>,
    #[serde(rename = "tms/bogus", skip_serializing_if = "Option::is_none")]
    tms_bogus: Option<Value>,
    exp: Value,
}

pub async fn make_auth_token(
    db_pool: &PgPool,
    client_id: &String,
    claims: HashMap<String, Value>,
) -> Result<String> {
    let mut tx = db_pool.begin().await?;
    let http_config = db_get_http_config(&mut tx).await?;
    let jwt_config = db_get_jwt_config(&mut tx).await?;
    let client = db_get_client_by_id(&mut tx, client_id).await?;
    let kid = &client.kid;
    let keys = db_get_key_by_id(&mut tx, &kid).await?;
    tx.commit().await?;

    let issuer = http_config.base_url;
    let subject = get_string_claim(&claims, CLAIM_SUB).await?;
    let idp = get_string_claim(&claims, CLAIM_IDP).await?;
    let tms_subject = format!("{0}@{1}", &subject, &idp);
    let tms_username = tms_subject.clone();

    let jwt_expiration_minutes = jwt_config.default_expiration_minutes.parse()?;
    let expiration = SystemTime::now() + Duration::from_mins(jwt_expiration_minutes);

    let tms_token_clams = TmsTokenClaims {
        jti: Value::from(Uuid::new_v4().to_string()),
        iss: Value::from(issuer),
        sub: Value::from(tms_subject),
        tms_token_type: Value::from("access"),
        tms_username: Value::from(tms_username),
        tms_client_id: Value::from(client_id.clone()),
        tms_grant_type: Value::from("password"),
        tms_account_type: Value::from("user"),
        tms_name: claims.get(CLAIM_NAME).map(|value| (*value).clone()),
        tms_idp_display_name: claims
            .get(CLAIM_IDP_DISPLAY_NAME)
            .map(|value| (*value).clone()),
        tms_organization: claims.get(CLAIM_ORGANIZATION).map(|value| (*value).clone()),
        tms_bogus: claims.get(CLAIM_BOGUS).map(|value| (*value).clone()),
        exp: Value::from(expiration.duration_since(UNIX_EPOCH)?.as_secs()),
    };

    // TODO: add kid, alg, and jti in header ... maybe other stuff?
    JwtEncoderBuilder::builder(
        tms_token_clams,
        keys.jwt_private_key.as_bytes(),
        DEFAULT_ALGORITHM,
        kid,
    )
    .encode()
    .await
}
async fn get_string_claim(claims: &HashMap<String, Value>, name: &str) -> Result<String> {
    let value = claims.get(name).ok_or(Unauthorized(format!(
        "Unable to find '{0}' claim in identity token",
        name
    )))?;

    let string_slice_value = value
        .as_str()
        .ok_or(Unauthorized(format!("Claim '{0}' is not a string", name)))?;
    Ok(String::from(string_slice_value))
}
