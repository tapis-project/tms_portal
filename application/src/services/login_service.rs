use crate::db::identity_provider_dao::{
    db_get_login_provider_by_id, db_get_login_providers,
};
use crate::db::keys_dao::db_get_key_by_id;
use crate::services::globus_token_provider::GlobusTokenProvider;
use tms_lib::utils::service_error::{ ServiceError, ServiceError::{BadRequest, Unauthorized}};
use crate::services::token_provider::TokenProvider;
use crate::utils::oauth2_authorization_code_utils::{decode_access_token, get_token_for_provider, OAuth2State};
use anyhow::{Context, Result};
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use crate::db::issued_tokens_dao::db_revoke_token;
use crate::obj_model::identity_provider::{IdentityProvider, IdentityProviderType};
use crate::obj_model::login::WhoAmIResponse;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{get_tms_token_claims, make_auth_token, TmsTokenClaims};
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
pub async fn get_identity_providers(pool: &PgPool) -> Result<HashSet<IdentityProvider>> {
    let mut tx = pool.begin().await?;
    let idps = db_get_login_providers(&mut tx).await?;
    tx.commit().await?;

    let mut idp_result = HashSet::new();
    idps.iter().for_each(|idp| {
        idp_result.insert(idp.clone().into());
    });
    Ok(idp_result)
}

pub async fn handle_callback(
    db_pool: &PgPool,
    state: &String,
    code: &String,
    cookie_state: &String,
) -> Result<(String, i64)> {
    if !cookie_state.eq(state) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state:OAuth2State = decode_state(db_pool, state)
        .await
        .context("Unable to decode state query param")?;

    let mut tx = db_pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    let configuration = Configuration::get(db_pool).await?;
    tx.commit().await?;


    let token:AuthorizationCodeResponse =
        get_token_for_provider(&idp, &configuration.http_config.get_identity_provider_callback_url(), code).await?;

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    // TODO: get iss from config
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));

    let tms_token_claims = get_tms_token_claims(&configuration, &decoded_state.client_id, &idp.id, &idp.identity_provider_type, &claims).await?;
    let tms_token = make_auth_token(db_pool, &tms_token_claims).await?;
    Ok((tms_token, tms_token_claims.get_expires_in()?))
}

pub async fn logout(db_pool: &PgPool, token:&String) -> Result<()> {
    let mut tx = db_pool.begin().await?;
    db_revoke_token(&mut tx, &token).await?;
    tx.commit().await?;
    Ok(())
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
