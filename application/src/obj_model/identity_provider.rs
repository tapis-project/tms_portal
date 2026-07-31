use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::db::identity_provider_dao::IdentityProviderType;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct IdentityProvider {
    pub uuid: Option<Uuid>,
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub identity_redirect_url: String,
    pub oauth2_token_url: String,
    pub oauth2_jwks_url: Option<String>,
    pub oidc_user_info_url: Option<String>,
    pub oauth2_public_key: Option<String>,
    pub scope: Option<String>,
    pub identity_provider_type: IdentityProviderType,
    pub supports_login: bool,
    pub supports_resources: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

