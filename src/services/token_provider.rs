use crate::models::login_api::WhoAmIResponse;
use crate::services::login_service::TmsTokenClaims;
use crate::services::service_error::AppError;

pub trait TokenProvider {
    fn whoami(&self, tms_token_claims: &TmsTokenClaims)
    -> anyhow::Result<WhoAmIResponse, AppError>;
}
