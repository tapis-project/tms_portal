use crate::models::general_api::{TmsResponse, TmsResponseBuilder};
use crate::models::resource_api::{GetResourceProviderResponse, ResourceProviderAuthorizeRequest};
use crate::services::login_service::{decode_state, handle_callback};
use crate::services::resource_service::{
    get_authenticate_redirect_info, get_resource_provider_token, get_resource_providers,
};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Unauthorized};
use crate::utils::oauth2_authorization_code_utils::{get_token_claims, AuthCodeQueryParams};
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
const RP_COOKIE_PATH: &str = "resource/provider/callback";

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
        get_authenticate_redirect_info(&app_state.db_pool, &client_id_string, &request.provider_id)
            .await?;
    // state cookies are stored in a cookie under RP_COOKIE_PATH named for the resource provider id
    let updated_jar = jar.add(
        Cookie::build((
            request.provider_id.clone(),
            authorize_info.encoded_state.clone(),
        ))
        .path(RP_COOKIE_PATH)
        .http_only(true),
    );

    let cookie = Cookie::build((request.provider_id, authorize_info.encoded_state))
        .path(RP_COOKIE_PATH)
        .http_only(true);
    tracing::warn!("{:?}", cookie);

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
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<TmsResponse<GetResourceProviderResponse>, AppError> {
    let decoded_state = decode_state(&app_state.db_pool, &query_params.state).await?;
    let resource_provider_id = decoded_state.idp_id;

    let thing = get_resource_provider_token(
        &app_state.db_pool,
        &resource_provider_id,
        &query_params.code,
    )
    .await?;

    // // TODO:  Add this check when you can verify it works (needs UI)
    //     let Some(state_cookie) = jar.get(&resource_provider_id) else {
    //         return Err(Unauthorized("No state cookies were found".to_string()).into());
    //     };
    //TODO:  be sure your checking the state
    //     let token = handle_callback(
    //         &app_state.db_pool,
    //         &query_params.state,
    //         &query_params.code,
    // //        &state_cookie.value().to_owned(),
    // // TODO:  remove the line below and re-add the line above3 when cookie check is in
    //         &query_params.state
    //     )
    //     .await?;

    // let c = Cookie::build((crate::routes::login::TOKEN_COOKIE_NAME, token))
    //     .path(crate::routes::login::ROOT_COOKIE_PATH)
    //     .http_only(false)
    //     .secure(true)
    //     .build();
    // let updated_jar = jar.clone().add(c);
    //
    // let decoded_state = decode_state(&app_state.db_pool, &state_cookie.value().to_owned()).await?;
    // let headers: HashMap<String, String> =
    //     HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)].into_iter());
    // Ok((
    //     updated_jar,
    //     TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
    //         .headers(headers)
    //         .build(),
    // ))
    Ok(TmsResponse::builder(StatusCode::OK).build())
}
