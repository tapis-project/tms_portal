use crate::db::allowed_redirects_dao::db_get_allowed_redirect;
use crate::db::config_dao::db_get_http_config;
use crate::db::idp_dao::db_get_idp_by_id;
use crate::models::api::TmsResponse;
use crate::models::oauth2::AuthorizeByIdpRequest;
use crate::routes::auth::STATE_COOKIE_NAME;
use crate::services::oauth_service::{encode_state, OAuthState};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal};
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{debug_handler, Form, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::signature::rand_core::{OsRng, RngCore};
use std::collections::HashMap;
use std::time::SystemTime;
use url::Url;

const ROOT_COOKIE_PATH: &str = "/";
const CLIENT_ID_TMS: &str = "tms";
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/login", get(login_handler))
}

#[debug_handler]
pub async fn login_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let client_id_string = String::from(CLIENT_ID_TMS);
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &form_data.idp_id).await;

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    let _ = db_get_allowed_redirect(&mut tx, &client_id_string, &form_data.redirect_uri).await?;
    tx.commit().await?;

    match idp {
        Ok(idp) => {
            let oauth_state = OAuthState {
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
            let callback_url = &http_config.get_callback_url();
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
