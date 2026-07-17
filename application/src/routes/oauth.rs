/*
- This file handles the oauth authrization routes
 */
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use anyhow::Context;
use axum::{debug_handler, Router};
use axum::extract::{State, Query};
use axum::routing::{get};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::{BadRequest, Unauthorized};
use crate::AppState;
use crate::db::identity_provider_dao::db_get_login_provider_by_id;
use crate::models::app_error::AppError;
use crate::models::tms_response::TmsResponse;
use crate::services::login_service::{decode_access_token, decode_state, get_login_provider_token, make_auth_token};
use crate::services::oauth2_service::{get_client, get_login_identity_provider, get_login_redirect_location, get_state, token_from_code};
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME};

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

impl TryFrom<&str> for GrantType {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v.eq_ignore_ascii_case("authorization_code") => Ok(GrantType::AuthorizationCode),
            v if v.eq_ignore_ascii_case("refresh_token") => Ok(GrantType::RefreshToken),
            _ => Err(BadRequest(format!("Unknown GrantType {}", value))),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum ResponseType {
    #[serde(rename = "code")]
    Code,
}

impl TryFrom<&str> for ResponseType {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            v if v.eq_ignore_ascii_case("code") => Ok(ResponseType::Code),
            _ => Err(BadRequest(format!("Unknown ResponseType {}", value))),
        }
    }
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseType::Code => write!(f, "code"),
        }
    }
}

#[derive(Debug,Deserialize)]
struct OAuthAuthorizeRequest {
    response_type: ResponseType,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OAuthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: GrantType,
    pub redirect_uri: String,
    pub code: String,
    pub refresh_token: String,
}

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/authorize", get(authorize_handler))
        .route("/oauth2/tokens", get(tokens_handler))
        .route("/oauth2/callback", get(callback_handler))
}
#[debug_handler]
pub async fn authorize_handler(State(app_state): State<AppState>, jar:CookieJar,
                            query_params: Query<OAuthAuthorizeRequest>) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    match query_params.response_type {
        ResponseType::Code => {
            let idp = get_login_identity_provider(&app_state.db_pool).await?;
            let client = get_client(&app_state.db_pool, &query_params.client_id).await?;
            let encoded_state = get_state(&app_state.db_pool, &query_params.client_id,
                                          &query_params.redirect_uri, idp.id).await?;
            let location = get_login_redirect_location(&app_state.db_pool, &query_params.client_id,
                                                       &query_params.redirect_uri, &encoded_state).await?;
            let updated_jar = jar.add(
                Cookie::build((STATE_COOKIE_NAME, encoded_state))
                    .path(ROOT_COOKIE_PATH)
                    .http_only(true),
            );

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            let creds = format!("{}:{}", client.id, client.secret);
            let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
            headers.insert("Authorization".to_string(), authorization);

            Ok((
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ))
        }
    }
}

#[debug_handler]
pub async fn tokens_handler(State(app_state): State<AppState>,
                            query_params: Query<OAuthTokenRequest>) -> anyhow::Result<String, AppError> {
    match query_params.grant_type {
        GrantType::AuthorizationCode => {
            let value = token_from_code(&app_state.db_pool, &query_params.client_id,
                                        &query_params.client_secret, &query_params.code,
                                        &query_params.redirect_uri).await?;
            Ok(value)
        },

        _ => {
            Err(BadRequest("Invalid grant type".to_string()).into())
        }
    }
}

#[debug_handler]
pub async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    //
    // // TODO:
    // Start here
    todo!()
}


#[cfg(test)]
mod tests {
    use crate::routes::oauth::{GrantType, ResponseType};

    #[test]
    fn test_grant_type_from_str() {
        assert_eq!(GrantType::try_from("authorization_code").unwrap(), GrantType::AuthorizationCode);
        assert_eq!(GrantType::try_from("AuthoRization_code").unwrap(), GrantType::AuthorizationCode);
        assert_eq!(GrantType::try_from("REFRESH_TOKEN").unwrap(), GrantType::RefreshToken);
        assert_eq!(GrantType::try_from("refresh_token").unwrap(), GrantType::RefreshToken);
        let result = GrantType::try_from("bogus_value");
        assert!(result.is_err());
    }
    #[test]
    fn test_response_type_from_str() {
        assert_eq!(ResponseType::try_from("code").unwrap(), ResponseType::Code);
        assert_eq!(ResponseType::try_from("CODE").unwrap(), ResponseType::Code);
        assert_eq!(ResponseType::try_from("CodE").unwrap(), ResponseType::Code);
        let result = ResponseType::try_from("bogus_value");
        assert!(result.is_err());
    }
}