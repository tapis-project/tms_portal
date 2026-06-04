use crate::db::allowed_redirects_dao::db_get_allowed_redirect;
use crate::db::config_dao::db_get_http_config;
use crate::db::identity_provider_dao::db_get_idp_by_id;
use crate::models::general_api::TmsResponse;
use crate::models::login_api::{
    AuthorizeByIdpRequest, IdentityProvider, TokenResponse, WhoAmIResponse,
};
use crate::services::login_service::{
    decode_state, encode_state, get_identity_providers, handle_callback, whoami,
};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use crate::utils::oauth2_authorization_code_utils::{AuthCodeQueryParams, OAuth2State};
use crate::AppState;
use axum::extract::{Query, State};
use axum::http::header::LOCATION;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{debug_handler, Form, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

const ROOT_COOKIE_PATH: &str = "/";
const CLIENT_ID_TMS: &str = "tms";
const TOKEN_COOKIE_NAME: &str = "tmstoken";
pub const STATE_COOKIE_NAME: &str = "state_cookie";

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/login", get(login_handler))
        .route("/login/whoami", get(whoami_handler))
        .route("/login/callback", get(callback_handler))
        .route("/login/idp", get(get_idp_handler))
}

#[debug_handler]
pub async fn login_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    let client_id_string = String::from(CLIENT_ID_TMS);
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &form_data.idp_id).await;

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    let _ = db_get_allowed_redirect(&mut tx, &client_id_string, &form_data.redirect_uri).await?;
    tx.commit().await?;

    match idp {
        Ok(idp) => {
            let oauth_state = OAuth2State {
                client_id: client_id_string,
                idp_id: form_data.idp_id.clone(),
                redirect_uri: form_data.redirect_uri.clone(),
                exp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_secs()
                    + 300000,
            };

            let encoded_state = match encode_state(&app_state.db_pool, oauth_state).await {
                Ok(state_string) => state_string,
                Err(error) => return Err(Internal(error.to_string()).into()),
            };

            let mut tx = app_state.db_pool.begin().await?;
            let http_config = db_get_http_config(&mut tx).await?;
            tx.commit().await?;
            let callback_url = &http_config.get_identity_provider_callback_url();
            let mut nonce = [0u8; 12];
            OsRng.fill_bytes(&mut nonce);
            let nonce_slice = BASE64_STANDARD.encode(nonce);
            let mut query_params = vec![
                ("response_type", "code"),
                ("client_id", &idp.client_id),
                ("redirect_uri", callback_url),
                ("state", &encoded_state),
                ("nonce", &nonce_slice),
                ("access_type", "offline"),
            ];

            if let Some(scope) = &idp.scope {
                query_params.push(("scope", scope.as_str()))
            }

            // TODO:  make a real nonce
            let location = Url::parse_with_params(&idp.identity_redirect_url, query_params)?;

            let updated_jar = jar.add(
                Cookie::build((STATE_COOKIE_NAME, encoded_state))
                    .path(ROOT_COOKIE_PATH)
                    .http_only(true),
            );

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            let creds = format!("{}:{}", idp.client_id, idp.client_secret);
            let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
            headers.insert("Authorization".to_string(), authorization);

            Ok((
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            ))
        }

        Err(error) => Err(BadRequest(error.to_string()).into()),
    }
}

pub async fn whoami_handler(
    State(app_state): State<AppState>,
    TypedHeader(bearer): TypedHeader<Authorization<Bearer>>,
) -> anyhow::Result<TmsResponse<WhoAmIResponse>, AppError> {
    let token = &String::from(bearer.token());
    let whoami_response = whoami(&app_state.db_pool, token).await?;
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(whoami_response)
        .build())
}

#[debug_handler]
pub async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<TokenResponse>), AppError> {
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
    let c = Cookie::build((crate::routes::login::TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .http_only(false)
        .secure(true)
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

#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> anyhow::Result<TmsResponse<HashSet<IdentityProvider>>, AppError> {
    let idp_result = get_identity_providers(&app_state.db_pool).await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(idp_result)
        .build())
}
