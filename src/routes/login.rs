use crate::db::allowed_redirects_dao::db_get_allowed_redirect;
use crate::db::config_dao::db_get_http_config;
use crate::db::idp_dao::db_get_idp_by_id;
use crate::models::api::TmsResponse;
use crate::models::oauth2::AuthorizeByIdpRequest;
use crate::services::oauth_service::{encode_state, OAuthState};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal};
use crate::utils::basic_auth::basic_auth_is_authorized;
use crate::AppState;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{debug_handler, Form, Router};
use axum_extra::extract::CookieJar;
use axum_extra::headers::authorization::Basic;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use std::collections::HashMap;
use std::time::SystemTime;
use url::Url;

const STATE_COOKIE_PATH: &str = "tms/oauth2/state_cookie";
pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/login", get(login_handler))
}

#[debug_handler]
pub async fn login_handler(
    State(app_state): State<AppState>,
    authorization_header: Option<TypedHeader<Authorization<Basic>>>,
    jar: CookieJar,
    headers: HeaderMap,
    form_data: Form<AuthorizeByIdpRequest>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    let (client_id, client_secret) = match authorization_header {
        Some(header) => (
            header.0.username().to_owned(),
            header.0.password().to_owned(),
        ),
        _ => ("tms".to_string(), "tms".to_string()),
    };
    basic_auth_is_authorized(&app_state.db_pool, &client_id, &client_secret).await?;
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &form_data.idp_id).await;
    let allowed_redirects =
        db_get_allowed_redirect(&mut tx, &client_id, &form_data.redirect_uri).await?;
    tx.commit().await?;

    match idp {
        Ok(idp) => {
            let oauth_state = OAuthState {
                client_id,
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

            let updated_jar = jar.add((STATE_COOKIE_PATH, encoded_state));

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
