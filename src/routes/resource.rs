use crate::models::general_api::TmsResponse;
use crate::models::resource_api::{GetResourceProviderResponse, ResourceProviderAuthorizeRequest};
use crate::services::resource_service::{get_authenticate_redirect, get_resource_providers};
use crate::services::service_error::AppError;
use crate::utils::oauth2_utils::{get_token_claims, AuthCodeQueryParams};
use crate::AppState;
use anyhow::Result;
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{debug_handler, extract, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::StatusCode;
use std::collections::HashMap;

const CLIENT_ID_TMS: &str = "tms";
const RP_COOKIE_PATH: &str = "/resource_provider/";

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/resource/provider", get(get_resource_provider_handler))
        .route("/resource/provider/authorize", post(authorize_handler))
        .route(
            "/resource/provider/callback",
            get(get_resource_provider_callback_handler),
        )
}

#[debug_handler]
pub async fn authorize_handler(
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    State(app_state): State<AppState>,
    jar: CookieJar,
    extract::Json(request): extract::Json<ResourceProviderAuthorizeRequest>,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    let client_id_string = String::from(CLIENT_ID_TMS);
    // validate token
    let token = &String::from(bearer.token());
    get_token_claims(&app_state.db_pool, &token);

    let authorize_info =
        get_authenticate_redirect(&app_state.db_pool, &client_id_string, &request.provider_id)
            .await?;
    // state cookies are stored in a cookie under RP_COOKIE_PATH named for the resource provider id
    let updated_jar = jar.add(
        Cookie::build((request.provider_id, authorize_info.encoded_state))
            .path(RP_COOKIE_PATH)
            .http_only(true),
    );

    let creds = format!(
        "{}:{}",
        authorize_info.client_id, authorize_info.client_secret
    );
    let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
    let mut headers = HashMap::new();
    headers.insert(
        "location".to_string(),
        authorize_info.identity_redirect_url.to_string(),
    );
    headers.insert("Authorization".to_string(), authorization);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}

#[debug_handler]
pub async fn get_resource_provider_handler(
    State(app_state): State<AppState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> Result<TmsResponse<GetResourceProviderResponse>, AppError> {
    // validate token
    let token = &String::from(bearer.token());
    get_token_claims(&app_state.db_pool, &token);

    let result = get_resource_providers(&app_state.db_pool).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}

#[debug_handler]
pub async fn get_resource_provider_callback_handler(
    State(app_state): State<AppState>,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<TmsResponse<GetResourceProviderResponse>, AppError> {
    Ok(TmsResponse::builder(StatusCode::IM_A_TEAPOT).build())
}
