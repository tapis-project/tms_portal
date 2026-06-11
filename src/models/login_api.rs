use crate::db::identity_provider_dao;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct AuthorizeByIdpRequest {
    pub idp_id: String,
    pub redirect_uri: String,
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

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Clone)]
pub struct IdentityProvider {
    pub id: String,
    pub name: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "oauth2TokenUrl")]
    pub oauth2_token_url: String,
    #[serde(rename = "userInfoUrl")]
    pub user_info_url: Option<String>,
}

pub type GetIdentityProviderResponse = HashSet<IdentityProvider>;

impl From<identity_provider_dao::IdentityProvider> for IdentityProvider {
    fn from(value: identity_provider_dao::IdentityProvider) -> Self {
        IdentityProvider {
            id: value.id.to_string(),
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
        }
    }
}
