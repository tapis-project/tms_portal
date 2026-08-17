use crate::obj_model::login::WhoAmIResponse;
use crate::services::token_provider::TokenProvider;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::TmsTokenClaims;

pub struct GlobusTokenProvider {}

impl TokenProvider for GlobusTokenProvider {
    fn whoami(
        &self,
        tms_token_claims: &TmsTokenClaims,
    ) -> anyhow::Result<WhoAmIResponse, AppError> {
        Ok(WhoAmIResponse {
            name: tms_token_claims.tms_name.clone(),
            username: tms_token_claims.tms_username.clone(),
            organization: tms_token_claims.tms_organization.clone(),
            idp_display_name: tms_token_claims.tms_idp_display_name.clone(),
        })
    }
}
