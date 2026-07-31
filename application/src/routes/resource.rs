use crate::models::tms_response::TmsResponse;
use crate::models::resource_api::{GetLinkedResourceProviderResponse, GetResourceProviderResponse, GetResourceResponse, Resource, ResourceProviderAuthorizeRequest, UnlinkResourceProviderResponse};
use crate::services::resource_service::{get_authenticate_redirect_info, get_linked_resource_providers, get_resource_provider_token, get_resource_providers, unlink_resource_provider};
use tms_lib::utils::service_error::ServiceError::Unauthorized;
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, ListResourceProviderRequestParams, OAuth2State, CLIENT_ID_TMS};
use crate::{AppState};
use anyhow::Result;
use axum::extract::{Path, Query, State};
use axum::http::header::LOCATION;
use axum::routing::{delete, get};
use axum::{debug_handler, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::StatusCode;
use std::collections::{HashMap, HashSet};
use std::string::ToString;
use uuid::Uuid;
use crate::models::app_error::AppError;
use crate::utils::jwt_utils::JwtValidator;
use crate::utils::state_utils::decode_state;

const RP_STATE_PREFIX:&str = "state_rp_id_";
const RP_COOKIE_PATH:&str = "/resources/providers";

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/resources/providers", get(list_resource_provider_handler))
        .route("/resources/{provider_id}/{provider_account_id}", get(get_resource_handler))
        .route("/resources/providers/links/{resource_provider_link_id}", delete(unlink_resource_provider_handler))
        .route("/resources/providers/links", get(get_linked_resource_provider_handler))
        .route("/resources/providers/authorize", get(authorize_handler))
        .route(
            "/resources/providers/callback",
            get(get_resource_provider_callback_handler),
        )
}

//#[require_token(security_context)]
#[debug_handler]
pub async fn authorize_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<ResourceProviderAuthorizeRequest>,
    JwtValidator(security_context): JwtValidator,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    // resource provider login will always be TMS client id
    let client_id = String::from(CLIENT_ID_TMS);

    let authorize_info = get_authenticate_redirect_info(
        &security_context.tms_identity,
        &app_state.db_pool,
        &client_id,
        &query_params.provider_id,
        &query_params.redirect_url,
        &query_params.state,
    )
    .await?;

    let updated_jar = jar.add(
        Cookie::build((
            get_rp_state_cookie_name(&query_params.provider_id),
            authorize_info.encoded_state.clone(),
        ))
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

fn get_rp_state_cookie_name(rp_name:&String) -> String {
    format!("{RP_STATE_PREFIX}{rp_name}")
}

#[debug_handler]
pub async fn list_resource_provider_handler(
    State(app_state): State<AppState>,
    query_params: Query<ListResourceProviderRequestParams>,
    JwtValidator(security_context): JwtValidator,
) -> Result<TmsResponse<GetResourceProviderResponse>, AppError> {
    let linked_only = query_params.linked_only.unwrap_or_else(|| false);
    let result = get_resource_providers(&security_context, &app_state.db_pool, &linked_only).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
#[debug_handler]
pub async fn unlink_resource_provider_handler (
    State(app_state): State<AppState>,
    Path((resource_provider_link_id)):Path<(i64)>,
    JwtValidator(security_context): JwtValidator,
) -> Result<TmsResponse<UnlinkResourceProviderResponse>, AppError> {
    let result = unlink_resource_provider(&security_context, &app_state.db_pool,
                                          &resource_provider_link_id).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
#[debug_handler]
pub async fn get_linked_resource_provider_handler (
    State(app_state): State<AppState>,
    JwtValidator(security_context): JwtValidator,
) -> Result<TmsResponse<GetLinkedResourceProviderResponse>, AppError> {
    let result = get_linked_resource_providers(&security_context, &app_state.db_pool).await?;
    Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
}
#[debug_handler]
pub async fn get_resource_handler(
    State(_app_state): State<AppState>,
    Path((provider_id, provider_account_id)): Path<(String, String)>,
    JwtValidator(_security_context): JwtValidator,
) -> Result<TmsResponse<GetResourceResponse>, AppError> {
    // Check that token is valid by getting claims
    let r1 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Stampede"),
        description: String::from("Stampede at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r2 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Vista"),
        description: String::from("Vista at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
        provider_name: String::from("TACC Resource Provider"),
    };

    let r3 = Resource {
        id: Uuid::new_v4().to_string(),
        name: String::from("Frontera"),
        description: String::from("Frontera at TACC"),
        provider_id: provider_id.clone(),
        provider_account_id: provider_account_id.clone(),
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
    let decoded_state:OAuth2State = decode_state(&app_state.db_pool, &query_params.state).await?;
    let resource_provider_id = decoded_state.idp_id;

    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(get_rp_state_cookie_name(&resource_provider_id).as_str()) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    if !&query_params.state.eq(state_cookie.value()) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state:OAuth2State = decode_state(&app_state.db_pool, &state_cookie.value().to_string()).await?;

    get_resource_provider_token(
        &app_state.db_pool,
        &resource_provider_id,
        &decoded_state.tms_identity,
        &query_params.code,
    )
    .await?;

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)].into_iter());

    let removal_cookie = Cookie::build(
        (get_rp_state_cookie_name(&resource_provider_id), String::from("")))
        .path(RP_COOKIE_PATH);
    let updated_jar = jar.remove(removal_cookie);

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}
