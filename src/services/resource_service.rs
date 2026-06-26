use crate::db::allowed_redirects_dao::db_get_allowed_redirect;
use crate::db::config_dao::db_get_http_config;
use crate::db::identity_provider_dao::{db_get_resource_provider_by_id, db_get_resource_providers};
use crate::models::resource_api::GetResourceProviderResponse;
use crate::services::login_service::encode_state;
use crate::services::service_error::ServiceError::{BadRequest, Internal};
use crate::utils::oauth2_authorization_code_utils::{get_token_for_provider, OAuth2State};
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::SystemTime;
use chrono::{Utc};
use serde_json::Value;
use url::Url;
use crate::db::resource_provider_account_logins::{db_add_or_update_resource_account_login};
use crate::services::service_error::AppError;
use crate::utils::jwt_utils::JwtDecoderBuilder;

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
    pub status: String,
    pub version: String,
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
    tms_identity: &String,
    db_pool: &PgPool,
    client_id: &String,
    provider_id: &String,
    redirect_url: &String,
) -> Result<ResourceProviderAuthorizeInfo, AppError> {
    let mut tx = db_pool.begin().await?;
    let rp = db_get_resource_provider_by_id(&mut tx, provider_id).await?;
    let _ = db_get_allowed_redirect(&mut tx, &client_id, &redirect_url).await?;
    tx.commit().await?;

    let oauth_state = OAuth2State {
        tms_identity: tms_identity.clone(),
        client_id: client_id.clone(),
        idp_id: rp.id,
        redirect_uri: redirect_url.clone(),
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
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    let nonce_slice = BASE64_STANDARD.encode(nonce);
    let redirect_uri = &http_config.get_resource_provider_callback_url();
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", &rp.client_id),
        ("redirect_uri", redirect_uri),
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
    tms_identity: &String,
    code: &String,
) -> Result<(), AppError> {
    let mut tx = db_pool.begin().await?;
    let rp = db_get_resource_provider_by_id(&mut tx, provider_id).await?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;

    let reponse: ResourceProviderAuthorizationCodeResponse =
        get_token_for_provider(&rp, &http_config.get_resource_provider_callback_url(), code)
            .await?;

//    Start here!!!
    // TODO:
    // these should be in a cookie or something
    // Also look at name of rp state cookie and rp token cookie, etc
    // also fill in all of the fields for the db_ad_or_update...
    let access_token = reponse.result.access_token;

    let decoded_token:Value = JwtDecoderBuilder::builder()
        .jwks_url(&rp.oauth2_jwks_url)
        .decode(&access_token.id_token).await?;

    let subject = match decoded_token.get("sub") {
        Some(subject) =>
            match subject.as_str() {
                Some(subject) => subject,
                None => return Err(BadRequest("Unable to retreive subject from access token".to_string()).into())
            }

        _ => return Err(BadRequest("Unable to retreive subject from access token".to_string()).into())
    };

    // I don't think we need this, right?
//    let refresh_token = reponse.result.refresh_token;

    let mut tx = db_pool.begin().await?;

    let tms_user_id = tms_identity.to_string();
    let resource_provider_account = subject;
    let resource_provider_id = provider_id.clone();
    let last_login = Utc::now();
    let enabled = false;

    db_add_or_update_resource_account_login(&mut tx, tms_user_id, resource_provider_account.to_string(),
                                            resource_provider_id, last_login, enabled).await?;
    tx.commit().await?;

    Ok(())
}
