use axum::extract::State;
use axum::{Json, Router};
use axum::routing::{post};
use http::StatusCode;
use serde::Deserialize;
use crate::AppState;
use crate::routes::api_obj_model::delegations::Delegation;
use crate::routes::api_obj_model::tms_response::TmsResponse;
use crate::services::delegation_service::add_delegation;
use crate::utils::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;

// params for tokens request
#[derive(Debug, Deserialize)]
pub struct AddDelegationRequest {
    resource_provider_id: String,
    resource_provider_account: String,
}

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/delegations", post(add_delegation_handler))
}
#[axum::debug_handler]
pub async fn add_delegation_handler(State(app_state): State<AppState>,
                                    JwtValidator(security_context): JwtValidator,
                                    Json(add_delegation_request): Json<AddDelegationRequest>,
                                    ) -> anyhow::Result<TmsResponse<Delegation>, AppError> {
    let delegation = add_delegation(&app_state.db_pool, &security_context.tms_identity,
        &security_context.client_id, &add_delegation_request.resource_provider_id,
        &add_delegation_request.resource_provider_account).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(delegation.into()).build())
}