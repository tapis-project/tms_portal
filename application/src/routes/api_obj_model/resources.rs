use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::obj_model;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLogin {
    pub id: i32,
    pub tms_identity: String,
    pub resource_provider_uuid: String,
    pub resource_provider_account: String,
    pub last_login: String,
    pub enabled: bool,
}


impl From<obj_model::resources::ResourceAccountLogin> for ResourceAccountLogin {
    fn from(value: obj_model::resources::ResourceAccountLogin) -> Self {
        ResourceAccountLogin {
            id: value.id,
            tms_identity: value.tms_identity.clone(),
            resource_provider_uuid: value.resource_provider_uuid.unwrap().to_string(),
            last_login: value.last_login.to_rfc3339(),
            enabled: value.enabled,
            resource_provider_account: value.resource_provider_account,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLink {
    pub id:i32,
    pub tms_identity:String,
    pub resource_provider_account:String,
    pub resource_provider_uuid:String,
    pub resource_provider_id:String,
    pub resource_provider_name:String,
    pub last_login:String,
    pub enabled:bool,
}
impl From<&obj_model::resources::ResourceAccountLink> for ResourceAccountLink {
    fn from(value: &obj_model::resources::ResourceAccountLink) -> Self {
        ResourceAccountLink {
            id: value.id,
            tms_identity: value.tms_identity.clone(),
            resource_provider_name: value.resource_provider_name.clone(),
            resource_provider_id: value.resource_provider_id.clone(),
            resource_provider_uuid: value.resource_provider_uuid.to_string(),
            last_login: value.last_login.to_rfc3339(),
            enabled: value.enabled,
            resource_provider_account: value.resource_provider_account.clone(),
        }
    }
}
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceProviderAuthorizeRequest {
    pub provider_id: String,
    pub redirect_url: String,
    pub state: Option<String>,
}
impl From<&obj_model::identity_provider::IdentityProvider> for ResourceProvider {
    fn from(value: &obj_model::identity_provider::IdentityProvider) -> Self {
        ResourceProvider {
            id: value.id.clone(),
            name: value.name.clone(),
            client_id: value.client_id.clone(),
            oauth2_token_url: value.oauth2_token_url.clone(),
            user_info_url: value.oidc_user_info_url.clone(),
        }
    }
}
