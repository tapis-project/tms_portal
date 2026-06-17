use crate::models::general_api::TmsResponse;
use crate::models::resource_api::{
    GetResourceProviderResponse, GetResourceResponse, Resource, ResourceProviderAuthorizeRequest,
};
use crate::services::login_service::decode_state;
use crate::services::resource_service::{
    get_authenticate_redirect_info, get_resource_provider_token, get_resource_providers,
};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::Unauthorized;
use crate::utils::oauth2_authorization_code_utils::{
    get_token_claims, AuthCodeQueryParams, CLIENT_ID_TMS, ROOT_COOKIE_PATH,
};
use crate::AppState;
use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::http::header::LOCATION;
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
use std::collections::{HashMap, HashSet};
use reqwest::header::HeaderMap;
use uuid::Uuid;

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/resources/providers", get(get_resource_provider_handler))
        .route("/resources/{provider_id}", get(get_resource_handler))
        .route("/resources/providers/authorize", get(authorize_handler))
        .route(
            "/resources/providers/callback",
            get(get_resource_provider_callback_handler),
        )
}

#[debug_handler]
pub async fn authorize_handler(
//    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    headers: HeaderMap,
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<ResourceProviderAuthorizeRequest>
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    // resource provider login will always be TMS client id
    let client_id = String::from(CLIENT_ID_TMS);
    // validate token
    let auth_header = headers.get(http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    let token = match auth_header {
        Some(header_value) => {
            match header_value.strip_prefix("Bearer ") {
                Some(token) => token.to_string(),
                _ => return Err(Unauthorized(String::from("No token found")).into())
            }
        }

        _ => return Err(Unauthorized(String::from("No token found")).into())
    };

    get_token_claims(&app_state.db_pool, &token).await?;

    let authorize_info = get_authenticate_redirect_info(
        &app_state.db_pool,
        &client_id,
        &query_params.provider_id,
        &query_params.redirect_url,
    )
    .await?;
    // state cookies are stored in a cookie under RP_COOKIE_PATH named for the resource provider id
    let updated_jar = jar.add(
        Cookie::build((
            query_params.provider_id.clone(),
            authorize_info.encoded_state.clone(),
        ))
        .path(ROOT_COOKIE_PATH)
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

    // Check that token is valid by getting claims
    let _token_claims = get_token_claims(&app_state.db_pool, token).await?;

    let result = get_resource_providers(&app_state.db_pool).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
#[debug_handler]
pub async fn get_resource_handler(
    State(app_state): State<AppState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
    Path(provider_id): Path<String>,
) -> Result<TmsResponse<GetResourceResponse>, AppError> {
    // validate token
    let token = &String::from(bearer.token());

    // Check that token is valid by getting claims
    let _token_claims = get_token_claims(&app_state.db_pool, token).await?;

    let r1 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Stampede"),
        description: String::from("Stampede at TACC"),
        provider_id: String::from("tacc"),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r2 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Vista"),
        description: String::from("Vista at TACC"),
        provider_id: String::from("tacc"),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r3 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Frontera"),
        description: String::from("Frontera at TACC"),
        provider_id: String::from("tacc"),
        provider_name: String::from("TACC Resource Provider"),
    };

    let result = HashSet::from([r1, r2, r3]);
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}

#[debug_handler]
pub async fn get_resource_provider_callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let decoded_state = decode_state(&app_state.db_pool, &query_params.state).await?;
    let resource_provider_id = decoded_state.idp_id;

    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(&resource_provider_id) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    if !&query_params.state.eq(state_cookie.value()) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    get_resource_provider_token(
        &app_state.db_pool,
        &resource_provider_id,
        &query_params.code,
    )
    .await?;

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)].into_iter());

    let updated_jar = jar.remove(Cookie::from(resource_provider_id));

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
