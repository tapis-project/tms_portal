use crate::db::client_dao::db_get_client_by_id;
use crate::db::config_dao::{db_get_http_config, db_get_jwt_config, db_get_state_key_id};
use crate::db::identity_provider_dao;
use crate::db::identity_provider_dao::{
    db_get_login_provider_by_id, db_get_login_providers, IdentityProviderType,
};
use crate::db::keys_dao::db_get_key_by_id;
use crate::models::login_api::{GetIdentityProviderResponse, WhoAmIResponse};
use crate::services::globus_token_provider::GlobusTokenProvider;
use crate::services::service_error::ServiceError::{BadRequest, Unauthorized};
use crate::services::service_error::{AppError, ServiceError};
use crate::services::token_provider::TokenProvider;
use crate::utils::jwt_utils::{JwtDecoderBuilder, JwtEncoderBuilder};
use crate::utils::oauth2_authorization_code_utils::{get_token_for_provider, OAuth2State};
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const DEFAULT_ALGORITHM: &str = "RS256";

const CLAIM_SUB: &str = "sub";
const CLAIM_IDP: &str = "identity_provider";
const CLAIM_NAME: &str = "name";
const CLAIM_IDP_DISPLAY_NAME: &str = "identity_provider_display_name";
const CLAIM_ORGANIZATION: &str = "organization";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TmsTokenClaims {
    pub jti: Value,
    pub iss: Value,
    pub sub: Value,
    #[serde(rename = "tms/token_type")]
    pub tms_token_type: Value,
    #[serde(rename = "tms/username")]
    pub tms_username: Value,
    #[serde(rename = "tms/client_id")]
    pub tms_client_id: Value,
    #[serde(rename = "tms/grant_type")]
    pub tms_grant_type: Value,
    #[serde(rename = "tms/account_type")]
    pub tms_account_type: Value,
    #[serde(rename = "tms/name", skip_serializing_if = "Option::is_none")]
    pub tms_name: Option<Value>,
    #[serde(
        rename = "tms/identity_provider_display_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub tms_idp_display_name: Option<Value>,
    #[serde(rename = "identity_provider_type")]
    pub tms_idp_provider: IdentityProviderType,
    #[serde(rename = "tms/organization", skip_serializing_if = "Option::is_none")]
    pub tms_organization: Option<Value>,
    pub exp: Value,
}

pub async fn get_identity_providers(pool: &PgPool) -> Result<GetIdentityProviderResponse> {
    let mut tx = pool.begin().await?;
    let idps = db_get_login_providers(&mut tx).await?;
    tx.commit().await?;

    let mut idp_result = GetIdentityProviderResponse::new();
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
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state = decode_state(pool, state)
        .await
        .context("Unable to decode state query param")?;
    dbg!(&decoded_state);

    let mut tx = pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    tx.commit().await?;

    let token = get_login_provider_token(pool, &idp, code).await?;
    dbg!(&token);

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    dbg!(&claims);
    make_auth_token(pool, &decoded_state.client_id, &idp, claims).await
}

pub async fn decode_state(pool: &PgPool, state_string: &String) -> Result<OAuth2State> {
    let mut tx = pool.begin().await?;
    let state_key = db_get_state_key_id(&mut tx).await?;
    let keys = db_get_key_by_id(&mut tx, &state_key.kid).await?;
    tx.commit().await?;
    // let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.jwt_public_key.as_bytes())?);

    JwtDecoderBuilder::builder()
        .public_key(&keys.jwt_public_key.as_bytes())
        .decode::<OAuth2State>(&state_string)
        .await
}

pub async fn encode_state(pool: &PgPool, oauth_state: OAuth2State) -> Result<String> {
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

pub async fn get_login_provider_token(
    db_pool: &PgPool,
    idp: &identity_provider_dao::IdentityProvider,
    code: &String,
) -> Result<AuthorizationCodeResponse> {
    let mut tx = db_pool.begin().await?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;

    get_token_for_provider(
        &idp,
        &http_config.get_identity_provider_callback_url(),
        code,
    )
    .await
}

pub async fn decode_access_token<T>(
    idp: &identity_provider_dao::IdentityProvider,
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

pub async fn make_auth_token(
    db_pool: &PgPool,
    client_id: &String,
    idp: &identity_provider_dao::IdentityProvider,
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
    let provider = idp.identity_provider_type.clone();
    let idp_id = get_string_claim(&claims, CLAIM_IDP).await?;
    let tms_subject = format!("{0}@{1}", &subject, &idp_id);
    let tms_username = tms_subject.clone();

    let jwt_expiration_minutes = jwt_config.default_expiration_minutes.parse()?;
    let expiration = SystemTime::now() + Duration::from_mins(jwt_expiration_minutes);

    let tms_token_claims = TmsTokenClaims {
        jti: Value::from(Uuid::new_v4().to_string()),
        iss: Value::from(issuer),
        sub: Value::from(tms_subject),
        tms_token_type: Value::from("access"),
        tms_username: Value::from(tms_username),
        tms_client_id: Value::from(client_id.clone()),
        tms_grant_type: Value::from("password"),
        tms_account_type: Value::from("user"),
        tms_name: claims.get(CLAIM_NAME).map(|value| (*value).clone()),
        tms_idp_provider: provider,
        tms_idp_display_name: claims
            .get(CLAIM_IDP_DISPLAY_NAME)
            .map(|value| (*value).clone()),
        tms_organization: claims.get(CLAIM_ORGANIZATION).map(|value| (*value).clone()),
        exp: Value::from(expiration.duration_since(UNIX_EPOCH)?.as_secs()),
    };

    // TODO: add kid, alg, and jti in header ... maybe other stuff?
    JwtEncoderBuilder::builder(
        tms_token_claims,
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

pub async fn whoami(db_pool: &PgPool, token: &String) -> anyhow::Result<WhoAmIResponse, AppError> {
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

    let idp_provider = &tms_token_claims.tms_idp_provider;
    let token_provider = match idp_provider {
        IdentityProviderType::Globus => GlobusTokenProvider {},
        _ => {
            return Err(ServiceError::Internal(format!(
                "Unsupported provider type {0}",
                idp_provider
            ))
            .into());
        }
    };

    token_provider.whoami(&tms_token_claims)
}
