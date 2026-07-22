use crate::db::identity_provider_dao::{
    db_get_login_provider_by_id, db_get_login_providers, IdentityProviderType,
};
use crate::db::keys_dao::db_get_key_by_id;
use crate::models::login_api::{GetIdentityProviderResponse, WhoAmIResponse};
use crate::services::globus_token_provider::GlobusTokenProvider;
use tms_lib::utils::service_error::{ ServiceError, ServiceError::{BadRequest, Unauthorized}};
use crate::services::token_provider::TokenProvider;
use crate::utils::oauth2_authorization_code_utils::{decode_access_token, get_token_for_provider};
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap};
use crate::models::app_error::AppError;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use crate::db::config_dao::db_get_http_config;
use crate::utils::jwt_utils::{make_auth_token, TmsTokenClaims};
use crate::utils::state_utils::decode_state;



#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    //    pub refresh_token_iat: u64,
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
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;


    let token:AuthorizationCodeResponse =
        get_token_for_provider(&idp, &http_config.get_identity_provider_callback_url(), code).await?;
    dbg!(&token);

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    dbg!(&claims);

    make_auth_token(pool, &decoded_state.client_id, &idp, claims).await
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
        .await.context("Error decoding JWT")?;

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
