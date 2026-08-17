use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tms_lib::utils::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use axum::extract::{FromRequestParts, State};
use axum_extra::extract::CookieJar;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use chrono::{DateTime, Utc};
use http::request::Parts;
use log::{info, trace};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use tracing::{enabled, Level};
use uuid::{Uuid};
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use crate::AppState;
use crate::db::config_dao::{db_get_jwt_config};
use crate::db::issued_tokens_dao::{db_get_token, db_insert_token};
use crate::db::keys_dao::db_get_key_by_id;
use crate::obj_model::identity_provider::IdentityProviderType;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;
use crate::utils::oauth2_authorization_code_utils::CLIENT_ID_TMS;

const DEFAULT_ALGORITHM: &str = "RS256";

const CLAIM_SUB: &str = "sub";
const CLAIM_IDP: &str = "identity_provider";
const CLAIM_NAME: &str = "name";
const CLAIM_IDP_DISPLAY_NAME: &str = "identity_provider_display_name";
const CLAIM_ORGANIZATION: &str = "organization";
const TMS_TOKEN_COOKIE_NAME: &str = "tmstoken";
pub trait Claims {
    fn get_string_claim(&self, name: &str) -> Result<String>;
}
pub type JwtClaims = HashMap<String, Value>;
pub struct SecurityContext {
    pub tms_identity: String,
    // the original token
    pub token: String,
    pub is_tms_client: bool,
}
pub struct JwtValidator(pub SecurityContext);
#[derive(Debug, Serialize, Deserialize)]
pub struct TmsTokenClaims {
    pub jti: Value,
    pub iss: Value,
    pub sub: Value,
    pub aud: Value,
    #[serde(rename = "tms/token_type")]
    pub tms_token_type: Value,
    #[serde(rename = "tms/username")]
    pub tms_username: Value,
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

impl TmsTokenClaims {
    pub fn get_jti(&self) -> Result<String> {
        get_string_from_value(&self.jti)
    }
    pub fn get_sub(&self) -> Result<String> {
        get_string_from_value(&self.sub)
    }

    pub fn get_expires_at(&self) -> Result<String> {
        let datetime = get_datetime_from_value(&self.exp)?;
        Ok(datetime.to_rfc3339())
    }
    pub fn get_expires_at_dt(&self) -> Result<DateTime<Utc>> {
        get_datetime_from_value(&self.exp)
    }
    pub fn get_expires_in(&self) -> Result<i64> {
        let datetime_now = Utc::now();
        let datetime_exp = get_datetime_from_value(&self.exp)?;
        let duration = datetime_exp - datetime_now;
        Ok(duration.num_seconds())
    }

    pub fn is_tms_client(&self) -> Result<bool> {
        if let (Some(client_id)) = self.aud.as_str() {
            Ok(CLIENT_ID_TMS == client_id)
        } else {
            // I'm not sure why this might happen, but it doesn't seem like a fatal error either,so
            // we'll just say it's not the tms client
            Ok(false)
        }
    }
}

async fn get_token_claims(db_pool: &PgPool, token: &String) -> Result<TmsTokenClaims> {
    let token_header = match decode_header(token) {
        Ok(header) => header,
        Err(e) => return Err(Unauthorized(e.to_string()).into()),
    };

    let mut tx = db_pool.begin().await?;
    let issued_token = db_get_token(&mut tx, token).await?;
    if(issued_token.is_revoked() || issued_token.is_expired()) {
        return Err(Unauthorized("Token is expired or revoked".to_string()).into());
    }
    let key = match token_header.kid {
        Some(kid) => db_get_key_by_id(&mut tx, &kid).await,
        None => return Err(BadRequest(String::from("Unable to find key for jwt")).into()),
    }?;
    tx.commit().await?;

    let tms_token_claims: TmsTokenClaims = match JwtDecoderBuilder::builder()
        .public_key(key.jwt_public_key.as_bytes())
        .decode(token)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            return Err(Unauthorized(e.to_string()).into());
        }
    };

    Ok(tms_token_claims)
}
impl FromRequestParts<AppState> for JwtValidator
where {
    type Rejection = AppError;

    async fn from_request_parts(req_parts: &mut Parts, app_state: &AppState) -> Result<Self, Self::Rejection> {
        let bearer = match TypedHeader::<Authorization<Bearer>>::from_request_parts(req_parts, &app_state).await {
            Ok(bearer) => bearer.token().to_string(),
            Err(_) => {
                let mut result:String = String::default();
                let Ok(app_state) = State::<AppState>::from_request_parts(req_parts, app_state).await;
                let Ok(jar) = CookieJar::from_request_parts(req_parts, &app_state).await;
                if let Some(token) = jar.get(TMS_TOKEN_COOKIE_NAME) {
                    result = token.value().to_string();
                };
                result
            },
        };

        if bearer.is_empty() {
            Err(Unauthorized(String::from("Unable to find TMS token")).into())
        } else {
            let jwt_claims = get_token_claims(&app_state.db_pool, &bearer).await?;
            let tms_identity = jwt_claims.get_sub()?;
            info!("Request uri: {} TMS Identity: {}", req_parts.uri, tms_identity);
            if(enabled!(Level::TRACE)) {
                req_parts.headers.iter().for_each(|(header_name, header_value)| {
                    trace!("Begin Headers");
                    trace!("Header: {} Value: {:?}", header_name, header_value);
                    trace!("End Headers");
                })
            }
            let is_tms_client = jwt_claims.is_tms_client()?;
            if(is_tms_client) {
                trace!("TMS Client connected");
            }
            Ok(JwtValidator(SecurityContext {
                tms_identity: tms_identity.to_string(),
                token: bearer.to_string(),
                is_tms_client,
                // we can add more to this if we need to
//                tms_token_claims: jwt_claims,
            }))
        }
    }
}

pub async fn make_auth_token(
    db_pool: &PgPool,
    // client_id: &String,
    // idp_id: &String,
    // idp_type: &IdentityProviderType,
    tms_token_claims: &TmsTokenClaims,
) -> Result<String> {
    let mut tx = db_pool.begin().await?;
    // let http_config = db_get_http_config(&mut tx).await?;
    let jwt_config = db_get_jwt_config(&mut tx).await?;
    let kid = &jwt_config.signing_key_kid;
    let keys = db_get_key_by_id(&mut tx, &kid).await?;
    tx.commit().await?;
    // let tms_token_claims = get_tms_token_claims(&http_config,
    //         &jwt_config, &client_id, &idp_id, &idp_type, &claims).await?;

    // TODO: add kid, alg, and jti in header ... maybe other stuff?
    let tms_token_string = JwtEncoderBuilder::builder(
        tms_token_claims,
        keys.jwt_private_key.as_bytes(),
        DEFAULT_ALGORITHM,
        kid,
    )
        .encode()
        .await?;

    let mut tx = db_pool.begin().await?;
    db_insert_token(&mut tx, &tms_token_string, &tms_token_claims.get_expires_at_dt()?).await?;
    tx.commit().await?;
    Ok(tms_token_string)
}
pub async fn get_tms_token_claims(configuration:&Configuration,
                                  client_id: &String,
                                  idp_id: &String,
                                  idp_type: &IdentityProviderType,
                                  claims: &JwtClaims,) -> Result<TmsTokenClaims> {
    let issuer = &configuration.http_config.base_url;
    let subject = claims.get_string_claim(CLAIM_SUB)?;
    let tms_subject = format!("{0}@{1}", &subject, &idp_id);
    let tms_username = tms_subject.clone();

    let jwt_expiration_minutes = configuration.jwt_config.default_expiration_minutes.parse()?;
    let expiration = SystemTime::now() + Duration::from_mins(jwt_expiration_minutes);
    let tms_token_claims = TmsTokenClaims {
        jti: Value::from(Uuid::new_v4().to_string()),
        iss: Value::from(issuer.clone()),
        sub: Value::from(tms_subject),
        aud: Value::from(client_id.clone()),
        tms_token_type: Value::from("access"),
        tms_username: Value::from(tms_username),
        tms_grant_type: Value::from("password"),
        tms_account_type: Value::from("user"),
        tms_name: claims.get(CLAIM_NAME).map(|value| (*value).clone()),
        tms_idp_provider: idp_type.clone(),
        tms_idp_display_name: claims
            .get(CLAIM_IDP_DISPLAY_NAME)
            .map(|value| (*value).clone()),
        tms_organization: claims.get(CLAIM_ORGANIZATION).map(|value| (*value).clone()),
        exp: Value::from(expiration.duration_since(UNIX_EPOCH)?.as_millis()),
    };
    Ok(tms_token_claims)
}

impl Claims for JwtClaims {
    fn get_string_claim(&self, name: &str) -> Result<String> {
        get_string_claim(self, name)
    }
}
fn get_string_claim(claims:&JwtClaims, name: &str) -> Result<String> {
    let value = claims.get(name).ok_or(Unauthorized(format!(
        "Unable to find '{0}' claim in identity token",
        name
    ))).context(format!("Claim name: {0}", name))?;
    get_string_from_value(value)
}
fn get_string_from_value(value:&Value) -> Result<String> {
    let string_slice_value = value
        .as_str()
        .ok_or(Unauthorized(format!("Value '{0}' is not a string", value)))?;
    Ok(String::from(string_slice_value))
}
fn get_datetime_from_value(value:&Value) -> Result<DateTime<Utc>> {
    let timestamp = value.as_i64()
        .ok_or(Unauthorized(format!("Value '{0}' is not a timestamp", value)))?;
    let Some(datetime) = DateTime::from_timestamp_millis(timestamp)
    else { return Err(Internal("Unable to determine expiration for token".to_string()).into())};
    Ok(datetime)
}



