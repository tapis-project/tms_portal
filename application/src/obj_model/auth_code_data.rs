use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::obj_model::identity_provider::IdentityProviderType;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AuthCodeData {
    pub auth_code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub idp_id: String,
    pub idp_type: IdentityProviderType,
    pub claims: Value,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}