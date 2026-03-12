use serde::Serialize;
use crate::db::idp_dao::Idp;
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