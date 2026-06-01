use crate::services::service_error::ServiceError;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct AuthCodeQueryParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeByIdpRequest {
    pub idp_id: String,
    pub redirect_uri: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2AuthorizeRequest {
    response_type: String,
    client_id: String,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2AuthorizeSuccess {
    pub(crate) code: String,
    pub(crate) state: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OAuth2Error {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
}

impl OAuth2Error {
    pub fn get_name(&self) -> String {
        match self {
            OAuth2Error::InvalidRequest => String::from("invalid_request"),
            OAuth2Error::UnauthorizedClient => String::from("unauthorized_client"),
            OAuth2Error::AccessDenied => String::from("access_denied"),
            OAuth2Error::UnsupportedResponseType => String::from("unsupported_response_type"),
            OAuth2Error::InvalidScope => String::from("invalid_scope"),
            OAuth2Error::ServerError => String::from("server_error"),
        }
    }
}

fn serialize_oauth2_error<S>(error: &OAuth2Error, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(error.get_name().as_str())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OAuth2AuthorizeError {
    #[serde(serialize_with = "serialize_oauth2_error")]
    pub(crate) error: OAuth2Error,
    pub(crate) error_description: Option<String>,
    pub(crate) error_uri: Option<String>,
    pub(crate) state: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OAuth2Response {
    #[serde(untagged)]
    Success(OAuth2AuthorizeSuccess),
    #[serde(untagged)]
    Error(OAuth2AuthorizeError),
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponseBody<T>
where
    T: Serialize,
{
    pub status: String,
    pub result: Option<T>,
}

pub fn internal_error_response(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoAmIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    pub username: Value,
    #[serde(skip_serializing_if = "Option::is_none", rename = "idpDisplayName")]
    pub idp_display_name: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum IdpProvider {
    Globus,
}

impl FromStr for IdpProvider {
    type Err = ServiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "globus" => Ok(IdpProvider::Globus),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", s))),
        }
    }
}
