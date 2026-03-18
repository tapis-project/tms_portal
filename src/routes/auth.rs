use crate::db::config_dao::get_state_cookie_path;
use crate::db::idp_dao::{get_idp_by_id, get_idps};
use crate::models::api::{Entity, TmsApiErrorResult, TmsResponse, TokenResponse};
use crate::models::oauth2::IdpResponse;
use crate::models::oauth2::{
    AuthCodeQueryParams, AuthorizationCodeResponse, AuthorizeByIdpRequest,
};
use crate::models::tms_internal::OAuthState;
use crate::services::oauth_service::{
    decode_access_token, decode_state, encode_state, exchange_code_for_token, make_auth_token,
};
use crate::AppState;
use axum::extract::State;
use axum::{debug_handler, extract::Query, routing::get, Form, Json, Router};
use axum_extra::extract::PrivateCookieJar;
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth2/callback", get(get_callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        .route("/oauth2/authorize", get(get_authorize_handler))
}
#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> TmsResponse<HashSet<IdpResponse>> {
    let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    let mut tx = app_state.db_pool.begin().await.unwrap();
    match get_idps(&mut tx).await {
        Ok(idps) => {
            tx.commit().await.unwrap();
            idps.iter().for_each(|idp| {
                idp_result.insert(idp.clone().into());
            });
            TmsResponse::builder(StatusCode::OK)
                .entity(Entity::Success(Json(idp_result)))
                .build()
        }

        Err(error) => {
            tx.rollback().await.unwrap();
            TmsResponse::builder(StatusCode::INTERNAL_SERVER_ERROR)
                .entity(Entity::Error(Json(TmsApiErrorResult {
                    message: error.to_string(),
                })))
                .build()
        }
    }
}

#[debug_handler]
pub async fn get_callback_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> TmsResponse<TokenResponse> {
    // get and decode state
    let state_string = match &(query_params.state) {
        Some(state_string) => state_string,
        None => {
            return TmsResponse::builder(StatusCode::UNAUTHORIZED)
                .entity(Entity::Error(Json(TmsApiErrorResult {
                    message: "Missing query parameter: State".to_string(),
                })))
                .build();
        }
    };

    let cookie_state = match jar.get(&get_state_cookie_path()) {
        Some(idp_cookie) if idp_cookie.value().eq(state_string) => idp_cookie.value().to_owned(),
        _ => {
            return TmsResponse::builder(StatusCode::UNAUTHORIZED)
                .entity(Entity::Error(Json(TmsApiErrorResult {
                    message: "No state cookies were found".to_string(),
                })))
                .build();
        }
    };
    dbg!(&cookie_state);

    let state = match decode_state(state_string).await {
        Ok(state) => state,
        Err(error) => {
            return TmsResponse::builder(StatusCode::UNAUTHORIZED)
                .entity(Entity::Error(Json(TmsApiErrorResult { message: error })))
                .build();
        }
    };
    dbg!(&state);

    let idp = match get_idp_by_id(&app_state.db_pool, &state.idp_id).await {
        Ok(idp) => idp,
        Err(_) => {
            return TmsResponse::builder(StatusCode::NOT_FOUND)
                .entity(Entity::Error(Json(TmsApiErrorResult {
                    message: "Idp was not found".to_string(),
                })))
                .build();
        }
    };

    let token: AuthorizationCodeResponse =
        match exchange_code_for_token(&idp, &query_params.code).await {
            Ok(token) => token,
            Err(error) => {
                return TmsResponse::builder(StatusCode::INTERNAL_SERVER_ERROR)
                    .entity(Entity::from(TmsApiErrorResult {
                        message: error.to_string(),
                    }))
                    .build();
            }
        };

    dbg!(&token);
    let claims: HashMap<String, Value> = match decode_access_token(&idp, &token.id_token).await {
        Ok(claims) => claims,
        Err(error) => {
            return TmsResponse::builder(StatusCode::NOT_FOUND)
                .entity(Entity::Error(Json(TmsApiErrorResult {
                    message: format!("Unable to decode id token: {}", error),
                })))
                .build();
        }
    };

    match make_auth_token(claims).await {
        Ok(token) => TmsResponse::builder(StatusCode::OK)
            .entity(Entity::Success(Json(TokenResponse { token })))
            .build(),
        Err(error) => TmsResponse::builder(StatusCode::INTERNAL_SERVER_ERROR)
            .entity(Entity::Error(Json(TmsApiErrorResult {
                message: format!("Error encoding token: {}", error),
            })))
            .build(),
    }
}
#[debug_handler]
pub async fn get_authorize_handler(
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> (PrivateCookieJar, TmsResponse<()>) {
    let idp = get_idp_by_id(&app_state.db_pool, &form_data.idp_id).await;

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

            let encoded_state = match encode_state(oauth_state).await {
                Ok(state_string) => state_string,
                Err(message) => {
                    return (
                        jar,
                        TmsResponse::builder(StatusCode::INTERNAL_SERVER_ERROR)
                            .entity(Entity::from(TmsApiErrorResult { message }))
                            .build(),
                    );
                }
            };

            // TODO:  make a real nonce
            let location = Url::parse_with_params(
                &idp.identity_redirect_url,
                [
                    ("response_type", "code"),
                    ("client_id", &idp.client_id),
                    ("redirect_uri", "http://localhost:8080/oauth2/callback"),
                    ("scope", &idp.scope),
                    ("state", &encoded_state),
                    ("nonce", "TODO: Add a real nonce"),
                ],
            )
            .unwrap();

            let updated_jar = jar.add((get_state_cookie_path(), encoded_state));

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            (
                updated_jar,
                TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
                    .headers(headers)
                    .build(),
            )
        }

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err),
            };

            (
                jar,
                TmsResponse::builder(StatusCode::BAD_REQUEST)
                    .entity(Entity::from(error))
                    .build(),
            )
        }
    }
}
