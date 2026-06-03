use crate::db::keys_dao::db_get_key_by_id;
use crate::services::login_service::TmsTokenClaims;
use crate::services::service_error::ServiceError::BadRequest;
use crate::utils::jwt_utils::JwtDecoderBuilder;
use anyhow::Result;
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

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
