/*
- This file handles the oauth authrization routes
 */
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use axum::{debug_handler, Json, Router};
use axum::extract::{State, Query};
use axum::routing::{get, post};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use http::header::LOCATION;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::{BadRequest, Unauthorized};
use crate::AppState;
use crate::models::app_error::AppError;
use crate::models::tms_response::TmsResponse;
use crate::services::oauth2_service::{get_access_token_from_code, authorize_code, TokenResponse, process_authorization_callback};
use crate::services::resource_service::AccessToken;
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME};

/*
Routes for this resource.  /oauth2/
*/
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/authorize", get(authorize_handler))
        .route("/oauth2/tokens", post(tokens_handler))
        .route("/oauth2/callback", get(callback_handler))
}

// Grant Type Enum for tokens request
#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

// Convert string to GrantType enum (ignore case)
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

// ResponseType enum for authorize endpoint
#[derive(Eq, PartialEq, Debug, Deserialize)]
pub enum ResponseType {
    #[serde(rename = "code")]
    Code,
}

// Convert string to ResponseType enum
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

// params for authorize request
#[derive(Debug,Deserialize)]
struct OAuthAuthorizeRequest {
    response_type: ResponseType,
    client_id: String,
    redirect_uri: String,
    scope: Option<String>,
    state: Option<String>,
}

// params for tokens request
#[derive(Debug, Deserialize)]
struct OAuthTokenRequest {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: GrantType,
    pub redirect_uri: String,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
}

// tokens response
#[derive(Debug, Serialize)]
struct OAuthTokenResponse {
    pub access_token: AccessToken
}

impl From<TokenResponse> for OAuthTokenResponse {
    fn from(value: TokenResponse) -> Self {
        let access_token = AccessToken {
            access_token: value.access_token,
            expires_in: value.expires_in,
            expires_at: value.expires_at,
            id_token: value.id_token,
            jti: value.jti,
        };

        OAuthTokenResponse {
            access_token
        }
    }
}

#[debug_handler]
async fn authorize_handler(State(app_state): State<AppState>, jar:CookieJar,
                            query_params: Query<OAuthAuthorizeRequest>) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Match exporession for response type to determine what kind of response is expected
    match query_params.response_type {
        // code - oauth authorization code flow
        ResponseType::Code => {
            let authorization_result = authorize_code(&app_state.db_pool, &query_params.state, &query_params.client_id, &query_params.redirect_uri).await?;

            let updated_jar = jar.add(
                Cookie::build((STATE_COOKIE_NAME, authorization_result.encoded_state))
                    .path(ROOT_COOKIE_PATH)
                    .http_only(true),
            );

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), authorization_result.location);

            Ok((
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ))
        }

        // TODO:  at some point we probably need to handle refresh tokens
    }
}


// hanlder for tokens request
#[debug_handler]
async fn tokens_handler(State(app_state): State<AppState>,
                        Json(oauth_token_request): Json<OAuthTokenRequest>) -> anyhow::Result<TmsResponse<OAuthTokenResponse>, AppError> {
    match oauth_token_request.grant_type {
        // handle authorization code flow
        GrantType::AuthorizationCode => {
            if let Some(code) = &oauth_token_request.code {
                let token_response = get_access_token_from_code(&app_state.db_pool, &oauth_token_request.client_id,
                                                       &oauth_token_request.client_secret, &code,
                                                       &oauth_token_request.redirect_uri).await?;
                Ok(TmsResponse::builder(StatusCode::OK)
                    .entity(OAuthTokenResponse::from(token_response)).build())
            } else {
                Err(BadRequest("Authorization code was not provided".to_string()).into())
            }
        },

        _ => {
            Err(BadRequest("Invalid grant type".to_string()).into())
        }
    }
}

// this should really only be called by the oauth redirect from an oauth provider (like globus, etc)
#[debug_handler]
async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    // handle the callback logic
    let authorization_callback_result = process_authorization_callback(
        &app_state.db_pool, &state_cookie.value().to_owned(), &query_params.state, &query_params.code).await?;

    let removal_cookie = Cookie::build(
        (STATE_COOKIE_NAME, String::from("")))
        .path(ROOT_COOKIE_PATH).http_only(true);

    let updated_jar = jar.remove(removal_cookie);

    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), authorization_callback_result.location)].into_iter());

    Ok((updated_jar, TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build()))
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