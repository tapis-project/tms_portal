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
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use http::header::LOCATION;
use http::StatusCode;
use serde::Deserialize;
use tms_lib::utils::service_error::ServiceError;
use tms_lib::utils::service_error::ServiceError::{BadRequest, Unauthorized};
use crate::AppState;
use crate::models::app_error::AppError;
use crate::models::tms_response::TmsResponse;
use crate::services::oauth2_service::{get_provider_token, generate_code_and_redirect,
                                      get_client, get_login_identity_provider,
                                      get_login_redirect_location, get_state, get_access_token_from_code};
use crate::services::resource_service::AccessToken;
use crate::utils::state_utils::decode_state;
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, ROOT_COOKIE_PATH, STATE_COOKIE_NAME, TOKEN_COOKIE_NAME};

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/authorize", get(authorize_handler))
        .route("/oauth2/tokens", post(tokens_handler))
        .route("/oauth2/callback", get(callback_handler))
}

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
    pub code: Option<String>,
    pub refresh_token: Option<String>,
}

struct OAuthTokenResponse {
    pub access_token: AccessToken
}

#[debug_handler]
async fn authorize_handler(State(app_state): State<AppState>, jar:CookieJar,
                            query_params: Query<OAuthAuthorizeRequest>) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    match query_params.response_type {
        ResponseType::Code => {
            // TODO can some of this be moved to the seervice class?  Probably!!
            let idp = get_login_identity_provider(&app_state.db_pool).await?;
            let client = get_client(&app_state.db_pool, &query_params.client_id).await?;
            let encoded_state = get_state(&app_state.db_pool, &query_params.client_id,
                                          &query_params.redirect_uri, &idp.id, &query_params.state).await?;
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
async fn tokens_handler(State(app_state): State<AppState>,
                        Json(oauth_token_request): Json<OAuthTokenRequest>) -> anyhow::Result<TmsResponse<AccessToken>, AppError> {
    match oauth_token_request.grant_type {
        GrantType::AuthorizationCode => {
            if let Some(code) = &oauth_token_request.code {
                let access_token = get_access_token_from_code(&app_state.db_pool, &oauth_token_request.client_id,
                                                       &oauth_token_request.client_secret, &code,
                                                       &oauth_token_request.redirect_uri).await?;
                Ok(TmsResponse::builder(StatusCode::OK)
                    .entity(access_token).build())
            } else {
                Err(BadRequest("Authorization code was not provided".to_string()).into())
            }
        },

        _ => {
            Err(BadRequest("Invalid grant type".to_string()).into())
        }
    }
}

#[debug_handler]
async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    // exchange code for token (state validated in handle_callback)
    let provider_token = get_provider_token(
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &state_cookie.value().to_owned(),
    )
        .await?;

    // // Build a new cookie and save it with the TMS token.
    // let c = Cookie::build((TOKEN_COOKIE_NAME, token))
    //     .path(ROOT_COOKIE_PATH)
    //     .http_only(false)
    //     .secure(true)
    //     .build();
    // let updated_jar = jar.clone().add(c);

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let decoded_state = decode_state(&app_state.db_pool, &state_cookie.value().to_owned()).await?;

    let location = generate_code_and_redirect(&app_state.db_pool, &decoded_state, &provider_token).await?;
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), String::from(location))].into_iter());

    // let updated_jar = updated_jar.remove(Cookie::from(STATE_COOKIE_NAME));

    Ok(//(
        // updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    )//)
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