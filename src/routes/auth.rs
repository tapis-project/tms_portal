use crate::models::api::{Entity, TmsResponse, TokenResponse};
use crate::models::oauth2::AuthCodeQueryParams;
use crate::services::oauth_service::decode_state;
use crate::services::oauth_service::{get_idps, handle_callback, IdpResponse};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::Unauthorized;
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use axum::http::header::LOCATION;
use axum::{debug_handler, extract::Query, routing::get, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use reqwest::StatusCode;
use std::collections::{HashMap, HashSet};
const TOKEN_COOKIE_NAME: &str = "tmstoken";
pub const STATE_COOKIE_NAME: &str = "state_cookie";
const ROOT_COOKIE_PATH: &str = "/";
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/callback", get(callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
}

#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> Result<TmsResponse<HashSet<IdpResponse>>, AppError> {
    let idp_result = get_idps(&app_state.db_pool).await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(Entity::Success(idp_result))
        .build())
}

#[debug_handler]
pub async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<(CookieJar, TmsResponse<TokenResponse>), AppError> {
    tracing::warn!(
        "OAuth2 callback query code: {0}, state: {1}",
        &query_params.code,
        &query_params.state
    );

    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    let token = handle_callback(
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &state_cookie.value().to_owned(),
    )
    .await?;
    let c = Cookie::build((TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .build();
    let updated_jar = jar.clone().add(c);

    let decoded_state = decode_state(&app_state.db_pool, &state_cookie.value().to_owned()).await?;
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)].into_iter());
    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
