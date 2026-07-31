use serde::{Serialize};
use serde_json::Value;

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