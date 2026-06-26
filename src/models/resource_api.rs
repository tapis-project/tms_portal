use crate::db::identity_provider_dao;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Clone)]
pub struct ResourceProvider {
    pub id: String,
    pub name: String,
    #[serde(rename = "clientId")]
    pub client_id: String,
    #[serde(rename = "oauth2TokenUrl")]
    pub oauth2_token_url: String,
    #[serde(rename = "userInfoUrl")]
    pub user_info_url: Option<String>,
}
pub type GetResourceProviderResponse = HashSet<ResourceProvider>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceProviderAuthorizeRequest {
    pub provider_id: String,
    pub redirect_url: String,
}

impl From<identity_provider_dao::IdentityProvider> for ResourceProvider {
    fn from(value: identity_provider_dao::IdentityProvider) -> Self {
        ResourceProvider {
            id: value.id,
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
        }
    }
}

#[derive(Debug, Hash, Serialize, Clone, PartialEq, Eq)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub provider_id: String,
    pub provider_account_id: String,
    pub provider_name: String,
}

pub type GetResourceResponse = HashSet<Resource>;
