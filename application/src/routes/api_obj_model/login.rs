use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::obj_model;

#[derive(Debug, Serialize)]
pub struct WhoAmIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    pub username: Value,
    #[serde(skip_serializing_if = "Option::is_none", rename = "idpDisplayName")]
    pub idp_display_name: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<Value>,
}

impl From<crate::obj_model::login::WhoAmIResponse> for WhoAmIResponse {
    fn from(value: crate::obj_model::login::WhoAmIResponse) -> Self {
        WhoAmIResponse {
            name: value.name,
            username: value.username,
            idp_display_name: value.idp_display_name,
            organization: value.organization,
        }
    }
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
    pub created: String,
    pub updated: String,
}

pub type GetIdentityProviderResponse = HashSet<IdentityProvider>;
impl From<obj_model::identity_provider::IdentityProvider> for IdentityProvider {
    fn from(value: obj_model::identity_provider::IdentityProvider) -> Self {
        IdentityProvider {
            id: value.id,
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
            created: value.created.to_rfc3339(),
            updated: value.updated.to_rfc3339(),
        }
    }
}
impl From<&obj_model::identity_provider::IdentityProvider> for IdentityProvider {
    fn from(value: &obj_model::identity_provider::IdentityProvider) -> Self {
        IdentityProvider {
            id: value.id.clone(),
            name: value.name.clone(),
            client_id: value.client_id.clone(),
            oauth2_token_url: value.oauth2_token_url.clone(),
            user_info_url: value.oidc_user_info_url.clone(),
            created: value.created.to_rfc3339().clone(),
            updated: value.updated.to_rfc3339().clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeByIdpRequest {
    pub idp_id: String,
    pub redirect_uri: String,
}