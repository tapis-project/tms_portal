use serde::{Deserialize, Serialize};

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
