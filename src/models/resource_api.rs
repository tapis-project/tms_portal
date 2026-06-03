use crate::db::resource_provider_dao;
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
}

impl From<resource_provider_dao::ResourceProvider> for ResourceProvider {
    fn from(value: resource_provider_dao::ResourceProvider) -> Self {
        ResourceProvider {
            id: value.id,
            name: value.name,
            client_id: value.client_id,
            oauth2_token_url: value.oauth2_token_url,
            user_info_url: value.oidc_user_info_url,
        }
    }
}
