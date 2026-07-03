use tms_lib::utils::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use anyhow::{Context, Result};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use axum::extract::{FromRequestParts, State};
use axum_extra::extract::CookieJar;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use axum_extra::TypedHeader;
use http::request::Parts;
use sqlx::PgPool;
use tms_lib::utils::app_error::AppError;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use crate::AppState;
use crate::db::keys_dao::db_get_key_by_id;
use crate::services::login_service::TmsTokenClaims;

pub struct SecurityContext {
    pub tms_identity: String,
//    pub tms_token_claims: TmsTokenClaims,
}
pub struct JwtValidator(pub SecurityContext);
const TMS_TOKEN_COOKIE_NAME: &str = "tmstoken";

async fn get_token_claims(db_pool: &PgPool, token: &String) -> Result<TmsTokenClaims> {
    let token_header = match decode_header(token) {
        Ok(header) => header,
        Err(e) => return Err(Unauthorized(e.to_string()).into()),
    };

    let mut tx = db_pool.begin().await?;
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
                if let Ok(app_state) = State::<AppState>::from_request_parts(req_parts, app_state).await {
                    if let Ok(jar) = CookieJar::from_request_parts(req_parts, &app_state).await {
                        if let Some(token) = jar.get(TMS_TOKEN_COOKIE_NAME) {
                            result = token.value().to_string();
                        };
                    };
                };

                result
            },
        };

        if bearer.is_empty() {
            Err(Unauthorized(String::from("Unable to find TMS token")).into())
        } else {
            let jwt_claims = get_token_claims(&app_state.db_pool, &bearer).await?;
            let tms_identity = jwt_claims.get_sub()?;
            dbg!(&tms_identity);
            Ok(JwtValidator(SecurityContext {
                tms_identity: tms_identity.to_string(),
                // we can add more to this if we need to
//                tms_token_claims: jwt_claims,
            }))
        }
    }
}
