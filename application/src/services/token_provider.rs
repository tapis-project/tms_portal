use crate::models::app_error::AppError;
use crate::models::login_api::WhoAmIResponse;
use crate::utils::jwt_utils::TmsTokenClaims;

pub trait TokenProvider {
    fn whoami(&self, tms_token_claims: &TmsTokenClaims)
    -> anyhow::Result<WhoAmIResponse, AppError>;
}
