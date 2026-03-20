use crate::db::idp_dao::Idp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationCodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token_iat: u64,
}

#[derive(Debug, Deserialize)]
pub struct AuthCodeQueryParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeByIdpRequest {
    pub idp_id: String,
}
//

#[derive(Debug, Serialize, Hash, Eq, PartialEq, Clone)]
pub struct IdpResponse {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub oauth2_tokent_url: String,
    pub user_info_url: String,
}

impl Into<IdpResponse> for Idp {
    fn into(self) -> IdpResponse {
        IdpResponse {
            id: self.id,
            name: self.name,
            client_id: self.client_id,
            oauth2_tokent_url: self.oauth2_token_url,
            user_info_url: self.user_info_url,
        }
    }
}
