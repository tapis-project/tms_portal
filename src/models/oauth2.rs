use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthCodeQueryParams {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeByIdpRequest {
    pub idp_id: String,
}
