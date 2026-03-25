use crate::db::config_dao::get_state_cookie_path;
use crate::db::idp_dao::db_get_idp_by_id;
use crate::models::api::{Entity, TmsResponse, TokenResponse};
use crate::models::oauth2::{AuthCodeQueryParams, AuthorizeByIdpRequest};
use crate::services::oauth_service::OAuthState;
use crate::services::oauth_service::{encode_state, get_idps, handle_callback, IdpResponse};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use crate::AppState;
use anyhow::Result;
use axum::extract::State;
use axum::{debug_handler, extract::Query, routing::get, Form, Router};
use axum_extra::extract::PrivateCookieJar;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::StatusCode;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/callback", get(get_callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        // .route("/oauth2/test", get(testit))
        .route("/oauth2/authorize", get(post_authorize_handler))
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
pub async fn get_callback_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> Result<TmsResponse<TokenResponse>, AppError> {
    tracing::warn!(
        "OAuth2 callback query code: {0}, state: {1}",
        &query_params.code,
        &query_params.state
    );

    let Some(state_cookie) = jar.get(&get_state_cookie_path()) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    let token = handle_callback(
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &state_cookie.value().to_owned(),
    )
    .await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(Entity::Success(TokenResponse { token }))
        .build())
}

#[debug_handler]
pub async fn post_authorize_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(PrivateCookieJar, TmsResponse<()>), AppError> {
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_idp_by_id(&mut tx, &form_data.idp_id).await;

    match idp {
        Ok(idp) => {
            tx.commit().await?;
            let oauth_state = OAuthState {
                idp_id: form_data.idp_id.clone(),
                exp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + 300000,
            };

            let encoded_state = match encode_state(oauth_state).await {
                Ok(state_string) => state_string,
                Err(error) => return Err(Internal(error.to_string()).into()),
            };

            let mut query_params = vec![
                ("response_type", "code"),
                ("client_id", &idp.client_id),
                ("redirect_uri", "http://localhost:8080/oauth2/callback"),
                ("state", &encoded_state),
                ("nonce", "TODO: Add a real nonce"),
                ("access_type", "offline"),
            ];

            if let Some(scope) = &idp.scope {
                query_params.push(("scope", scope.as_str()))
            }

            // TODO:  make a real nonce
            let location = Url::parse_with_params(&idp.identity_redirect_url, query_params)?;

            let updated_jar = jar.add((get_state_cookie_path(), encoded_state));

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

        Err(error) => {
            tx.rollback().await?;
            Err(BadRequest(error.to_string()).into())
        }
    }
}
