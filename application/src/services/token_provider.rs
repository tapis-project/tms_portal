use crate::models::app_error::AppError;
use crate::obj_model::login::WhoAmIResponse;
use crate::utils::jwt_utils::TmsTokenClaims;

pub trait TokenProvider {
    fn whoami(&self, tms_token_claims: &TmsTokenClaims)
    -> anyhow::Result<WhoAmIResponse, AppError>;
}
