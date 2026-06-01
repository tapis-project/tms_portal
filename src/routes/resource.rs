use crate::models::general_api::TmsResponse;
use crate::models::resource_api::GetResourceProviderResponse;
use crate::services::resource_service::get_resource_providers;
use crate::services::service_error::AppError;
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use axum::routing::get;
use axum::{debug_handler, Router};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use reqwest::StatusCode;

pub async fn router() -> Router<AppState> {
    Router::new().route("/resource/provider", get(get_resource_provider_handler))
}

#[debug_handler]
pub async fn get_resource_provider_handler(
    State(app_state): State<AppState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Result<TmsResponse<GetResourceProviderResponse>, AppError> {
    let token = &String::from(bearer.token());
    let result = get_resource_providers(&app_state.db_pool, token).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
