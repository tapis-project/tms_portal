use crate::db::config_dao::db_get_http_config;
use crate::db::resource_provider_dao::{db_get_resource_provider_by_id, db_get_resource_providers};
use crate::models::resource_api::GetResourceProviderResponse;
use crate::services::login_service::encode_state;
use crate::services::service_error::ServiceError::Internal;
use crate::utils::oauth2_utils;
use crate::utils::oauth2_utils::OAuth2State;
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::SystemTime;
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub expires_at: String,
    pub expires_in: u64,
    pub id_token: String,
    pub jti: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub refresh_token: String,
    pub expires_at: String,
    pub expires_in: u64,
    pub jti: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceProviderTokenResponse {
    pub access_token: AccessToken,
    pub refresh_token: RefreshToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceProviderAuthorizationCodeResponse {
    pub message: String,
    pub result: ResourceProviderTokenResponse,
}

pub struct ResourceProviderAuthorizeInfo {
    pub encoded_state: String,
    pub identity_redirect_url: Url,
    pub client_id: String,
    pub client_secret: String,
}
pub async fn get_resource_providers(db_pool: &PgPool) -> Result<GetResourceProviderResponse> {
    let mut tx = db_pool.begin().await?;
    let rps = db_get_resource_providers(&mut tx).await?;
    tx.commit().await?;

    let mut resource_provider_result = GetResourceProviderResponse::new();
    rps.iter().for_each(|rp| {
        resource_provider_result.insert(rp.clone().into());
    });
    Ok(resource_provider_result)
}
pub async fn get_authenticate_redirect_info(
    db_pool: &PgPool,
    client_id: &String,
    provider_id: &String,
) -> Result<ResourceProviderAuthorizeInfo> {
    let mut tx = db_pool.begin().await?;
    let rp = db_get_resource_provider_by_id(&mut tx, provider_id).await?;
    tx.commit().await?;

    let oauth_state = OAuth2State {
        client_id: client_id.clone(),
        idp_id: rp.id,
        redirect_uri: rp.identity_redirect_url.clone(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            + 300000,
    };

    let encoded_state = match encode_state(&db_pool, oauth_state).await {
        Ok(state_string) => state_string,
        Err(error) => return Err(Internal(error.to_string()).into()),
    };

    let mut tx = db_pool.begin().await?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;
    let callback_url = &http_config.get_identity_provider_callback_url();
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let nonce_slice = BASE64_STANDARD.encode(nonce);
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", &rp.client_id),
        ("redirect_uri", callback_url),
        ("state", &encoded_state),
        ("nonce", &nonce_slice),
        ("access_type", "offline"),
    ];

    if let Some(scope) = &rp.scope {
        query_params.push(("scope", scope.as_str()))
    }

    // TODO:  make a real nonce
    let identity_redirect_url = Url::parse_with_params(&rp.identity_redirect_url, query_params)?;

    Ok(ResourceProviderAuthorizeInfo {
        encoded_state,
        identity_redirect_url,
        client_id: rp.client_id,
        client_secret: rp.client_secret,
    })
}
pub async fn get_resource_provider_token(
    db_pool: &PgPool,
    provider_id: &String,
    code: &String,
) -> Result<()> {
    let mut tx = db_pool.begin().await?;
    let rp = db_get_resource_provider_by_id(&mut tx, provider_id).await?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;

    let result: ResourceProviderAuthorizationCodeResponse = get_token_for_provider(
        db_pool,
        &rp.oauth2_token_url,
        &rp.client_id,
        &rp.client_secret,
        &http_config.get_resource_provider_callback_url(),
        code,
    )
    .await?;
    tracing::debug!("get_resource_provider_token result: {:?}", result);
    Ok(())
}

async fn get_token_for_provider(
    db_pool: &PgPool,
    provider_token_url: &String,
    provider_client_id: &String,
    provider_client_secret: &String,
    callback_url: &String,
    code: &String,
) -> Result<ResourceProviderAuthorizationCodeResponse> {
    oauth2_utils::exchange_code_for_token(
        db_pool,
        provider_token_url,
        provider_client_id,
        provider_client_secret,
        callback_url,
        code,
    )
    .await
}
