use std::fmt::Display;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tms_lib::utils::service_error::ServiceError;

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
pub type ResourceProvider = IdentityProvider;
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum IdentityProviderType {
    Globus,
    TaccTapis,
}

impl FromStr for IdentityProviderType {
    type Err = ServiceError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        /*
                match value {
            v if v.eq_ignore_ascii_case("authorization_code") => Ok(GrantType::AuthorizationCode),
            v if v.eq_ignore_ascii_case("refresh_token") => Ok(GrantType::RefreshToken),
            _ => Err(BadRequest(format!("Unknown GrantType {}", value))),
        }

         */
        match value {
            idp_type if idp_type.eq_ignore_ascii_case("globus") => Ok(IdentityProviderType::Globus),
            idp_type if idp_type.eq_ignore_ascii_case("tacc_tapis") => Ok(IdentityProviderType::TaccTapis),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", value))),
        }
    }
}

impl Display for IdentityProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityProviderType::Globus => write!(f, "globus"),
            IdentityProviderType::TaccTapis => write!(f, "tacc_tapis"),
        }
    }
}
