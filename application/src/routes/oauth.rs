/*
- This file handles the oauth authrization routes
 */
use axum::{debug_handler, Form, Router};
use axum::extract::{State, Json};
use axum::routing::{get, post};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::BadRequest;
use crate::AppState;
use crate::models::app_error::AppError;

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

#[derive(Debug, Deserialize)]
pub struct OauthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub redirect_uri: String,
    pub code: String,
    pub refresh_token: String,
}

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/tokens", post(tokens_handler))
}

#[debug_handler]
pub async fn tokens_handler(State(app_state): State<AppState>,
                            jar: CookieJar,
                            Json(payload): Json<OauthTokenRequest>) -> anyhow::Result<String, AppError> {
    Ok(format!("Request: {:?}", payload))
}

#[cfg(test)]
mod tests {
    use crate::routes::oauth::GrantType;

    #[test]
    fn test_grant_type_from_str() {
        assert_eq!(GrantType::try_from("authorization_code").unwrap(), GrantType::AuthorizationCode);
        assert_eq!(GrantType::try_from("AuthoRization_code").unwrap(), GrantType::AuthorizationCode);
        assert_eq!(GrantType::try_from("REFRESH_TOKEN").unwrap(), GrantType::RefreshToken);
        assert_eq!(GrantType::try_from("refresh_token").unwrap(), GrantType::RefreshToken);
        let result = GrantType::try_from("bogus_value");
        assert!(result.is_err());
    }
}