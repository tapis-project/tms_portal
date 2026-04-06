use crate::db::config_dao::db_get_http_config;
use crate::db::idp_dao::db_get_idp_by_id;
use crate::models::api::{Entity, TmsResponse, TokenResponse};
use crate::models::oauth2::{AuthCodeQueryParams, AuthorizeByIdpRequest};
use crate::services::oauth_service::OAuthState;
use crate::services::oauth_service::{encode_state, get_idps, handle_callback, IdpResponse};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use crate::utils::basic_auth::basic_auth_is_authorized;
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::routing::post;
use axum::{debug_handler, extract::Query, routing::get, Form, Router};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::StatusCode;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

const CLIENT_ID_COOKIE_PATH: &str = "tms/oauth2/client_id";
const STATE_COOKIE_PATH: &str = "tms/oauth2/state_cookie";
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/callback", get(callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        .route("/oauth2/authorize", post(authorize_handler))
        .route("/oauth2/authorize", get(authorize_handler))
    //        .route("/oauth2/logon", get(logon_handler))
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
    jar: PrivateCookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<TmsResponse<TokenResponse>, AppError> {
    tracing::warn!(
        "OAuth2 callback query code: {0}, state: {1}",
        &query_params.code,
        &query_params.state
    );

    let Some(state_cookie) = jar.get(&STATE_COOKIE_PATH) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };
    let Some(client_id_cookie) = jar.get(CLIENT_ID_COOKIE_PATH) else {
        return Err(Unauthorized("No client_id cookies were found".to_string()).into());
    };

    let token = handle_callback(
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &client_id_cookie.value().to_owned(),
        &state_cookie.value().to_owned(),
    )
    .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(Entity::Success(TokenResponse { token }))
        .build())
}

#[debug_handler]
pub async fn authorize_handler(
    State(app_state): State<AppState>,
    authorization_header: Option<TypedHeader<Authorization<Basic>>>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(PrivateCookieJar, TmsResponse<()>), AppError> {
    let (client_id, client_secret) = match authorization_header {
        Some(header) => (
            header.0.username().to_owned(),
            header.0.password().to_owned(),
        ),
        _ => ("tms".to_string(), "tms".to_string()),
    };
    // let id = authorization_header.0.username().to_string();
    // let secret = authorization_header.0.password().to_string();
    basic_auth_is_authorized(&app_state.db_pool, &client_id, &client_secret).await?;
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &form_data.idp_id).await;
    tx.commit().await?;

    match idp {
        Ok(idp) => {
            let oauth_state = OAuthState {
                idp_id: form_data.idp_id.clone(),
                exp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
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
            let callback_url = &http_config.get_callback_url();

            let mut query_params = vec![
                ("response_type", "code"),
                ("client_id", &idp.client_id),
                ("redirect_uri", callback_url),
                ("state", &encoded_state),
                ("nonce", "TODO: Add a real nonce"),
                ("access_type", "offline"),
            ];

            if let Some(scope) = &idp.scope {
                query_params.push(("scope", scope.as_str()))
            }

            // TODO:  make a real nonce
            let location = Url::parse_with_params(&idp.identity_redirect_url, query_params)?;

            let updated_jar = jar
                .add((STATE_COOKIE_PATH, encoded_state))
                .add((CLIENT_ID_COOKIE_PATH, client_id));

            //           let jarresult = updated_jar.into_response();
            //           jarresult.headers().get("")
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

// use axum::response::Html;
// use std::fs::read_to_string;
// pub async fn logon_handler() -> (StatusCode, Html<String>) {
//     let page = read_to_string("./logon.html");
//     (StatusCode::OK, Html(page.unwrap()))
// }
