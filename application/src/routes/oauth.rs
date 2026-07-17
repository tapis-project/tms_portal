/*
- This file handles the oauth authrization routes
 */
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use axum::{debug_handler, Router};
use axum::extract::{State, Query};
use axum::routing::{get, post};
use axum_extra::extract::CookieJar;
use http::header::LOCATION;
use http::StatusCode;
use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::BadRequest;
use crate::AppState;
use crate::models::app_error::AppError;
use crate::models::tms_response::TmsResponse;
use crate::services::oauth2_service::handle_authorize_code_response;

#[derive(Eq, PartialEq, Debug)]
pub enum GrantType {
    AuthorizationCode,
    RefreshToken,
}

impl TryFrom<&str> for GrantType {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match(value) {
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
        match(value) {
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
    pub grant_type: String,
    pub redirect_uri: String,
    pub code: String,
    pub refresh_token: String,
}

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/authorize", get(authorize_handler))
        .route("/oauth2/tokens", get(tokens_handler))
}
#[debug_handler]
pub async fn authorize_handler(State(app_state): State<AppState>,
                            jar: CookieJar,
                            query_params: Query<OAuthAuthorizeRequest>) -> anyhow::Result<TmsResponse<()>, AppError> {
    match query_params.response_type {
        ResponseType::Code => {
            let redirect_location = handle_authorize_code_response(&app_state.db_pool, &query_params.client_id,
                                                                   &query_params.redirect_uri, &query_params.state,
                                                                   &query_params.scope).await?;

            // redirect browser back to the post-login page (taken from state - validated in login step).
            let headers: HashMap<String, String> =
                HashMap::from_iter(vec![(LOCATION.to_string(), String::from(redirect_location))].into_iter());

            return Ok(TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                .headers(headers)
                .build());
        }
    }
}

#[debug_handler]
pub async fn tokens_handler(State(app_state): State<AppState>,
                            jar: CookieJar,
                            query_params: Query<OAuthTokenRequest>) -> anyhow::Result<String, AppError> {
    Ok(format!("Request: {:?}", query_params))
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