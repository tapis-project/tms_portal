use tms_lib::utils::app_error::AppError;
use crate::models::login_api::WhoAmIResponse;
use crate::services::login_service::TmsTokenClaims;

pub trait TokenProvider {
    fn whoami(&self, tms_token_claims: &TmsTokenClaims)
    -> anyhow::Result<WhoAmIResponse, AppError>;
}
