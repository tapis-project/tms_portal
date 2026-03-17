use crate::db::idp_dao::{get_idp_by_id, get_idps};
use crate::models::api::{Entity, TmsApiErrorResult, TmsHttpResponse, TokenResponse};
use crate::models::oauth2::IdpResponse;
use crate::models::oauth2::{
    AuthCodeQueryParams, AuthorizationCodeResponse, AuthorizeByIdpRequest,
};
use crate::models::oidc_state::OAuthState;
use crate::models::tms_internal::TmsResult;
use crate::services::oauth_service::decode_state;
use crate::utils::jwt_utils::{decode_access_token, exchange_code_for_token, make_auth_token};
//use crate::services::oauth_service::decode_jwt;
use crate::AppState;
use axum::{extract::Query, routing::get, Form, Json, Router};
use axum_extra::extract::cookie::Key;
use axum_extra::extract::PrivateCookieJar;
use reqwest::StatusCode;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use url::Url;

pub fn router() -> Router {
    let state = AppState {
        // Generate a secure key
        //
        // TODO:  You probably don't wanna generate a new one each time the app starts though
        key: Key::generate(),
    };

    Router::new()
        .route("/oauth2/callback", get(get_callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        .route("/oauth2/authorize", get(get_authorize_handler))
        .with_state(state)
}
pub async fn get_idp_handler() -> TmsHttpResponse<HashSet<IdpResponse>> {
    let mut idp_result: HashSet<IdpResponse> = HashSet::new();
    match get_idps().await {
        Ok(idps) => {
            idps.iter().for_each(|idp| {
                idp_result.insert(idp.clone().into());
            });

            TmsHttpResponse::from((StatusCode::OK, Entity::Success(Json(idp_result))))
        }

        Err(error_string) => TmsHttpResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            Entity::Error(Json(TmsApiErrorResult {
                message: error_string,
            })),
        )),
    }
}

pub async fn get_callback_handler(
    jar: PrivateCookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> TmsHttpResponse<TokenResponse> {
    // get and decode state
    let state_string = match &(query_params.state) {
        Some(state_string) => state_string,
        None => {
            return TmsHttpResponse::from((
                StatusCode::UNAUTHORIZED,
                Entity::Error(Json(TmsApiErrorResult {
                    message: "Missing query parameter: State".to_string(),
                })),
            ));
        }
    };

    let cookie_state = match jar.get("oauth2/idp_cookie") {
        Some(idp_cookie) if idp_cookie.value().eq(state_string) => idp_cookie.value().to_owned(),
        _ => {
            return TmsHttpResponse::from((
                StatusCode::UNAUTHORIZED,
                Entity::Error(Json(TmsApiErrorResult {
                    message: "No state cookies were found".to_string(),
                })),
            ));
        }
    };
    dbg!(&cookie_state);

    let state = match decode_state(state_string).await {
        Ok(state) => state,
        Err(error) => {
            return TmsHttpResponse::from((
                StatusCode::UNAUTHORIZED,
                Entity::Error(Json(TmsApiErrorResult { message: error })),
            ));
        }
    };
    dbg!(&state);

    let idp = match get_idp_by_id(&state.idp_id).await {
        Ok(idp) => idp,
        Err(error) => {
            return TmsHttpResponse::from((
                StatusCode::NOT_FOUND,
                Entity::Error(Json(TmsApiErrorResult {
                    message: "Idp was not found".to_string(),
                })),
            ));
        }
    };

    // let params = [
    //     ("grant_type", "authorization_code"),
    //     (
    //         "client_id",
    //         "cilogon:/client_id/3d38b53c9709489136c9b68c8f769c99",
    //     ),
    //     (
    //         "client_secret",
    //         "_-9v-A023hVLquAlLjToWSQoF5XOwCKjG8i8QHtCN3K4c8fjF_ILSmf4ZekXafk0VC6q_T66WOntSUxgJLjN1Q",
    //     ),
    //     ("code", &query_params.0.code),
    // ];

    let token: TmsResult<AuthorizationCodeResponse> =
        exchange_code_for_token(&idp, &query_params.code).await;
    dbg!(&token);
    let claims: HashMap<String, Value> =
        match decode_access_token(&idp, &token.unwrap().id_token).await {
            Ok(claims) => claims,
            Err(error) => {
                return TmsHttpResponse::from((
                    StatusCode::NOT_FOUND,
                    Entity::Error(Json(TmsApiErrorResult {
                        message: format!("Unable to decode id token: {}", error),
                    })),
                ));
            }
        };

    match make_auth_token(claims).await {
        Ok(token) => TmsHttpResponse::from((
            StatusCode::OK,
            Entity::Success(Json(TokenResponse { token })),
        )),
        Err(error) => TmsHttpResponse::from((
            StatusCode::INTERNAL_SERVER_ERROR,
            Entity::Error(Json(TmsApiErrorResult {
                message: format!("Error encoding token: {}", error),
            })),
        )),
    }
}
pub async fn get_authorize_handler(
    jar: PrivateCookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> (PrivateCookieJar, TmsHttpResponse<()>) {
    let idp = get_idp_by_id(&form_data.idp_id).await;

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

            let encoded_state = match oauth_state.encode() {
                Ok(state_string) => {
                    println!("state: {:?}", state_string);
                    state_string
                }
                Err(error) => {
                    println!("error: {:?}", error);
                    return (
                        jar,
                        TmsHttpResponse::from((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Entity::from(error),
                        )),
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

            let updated_jar = jar.add(("oauth2/idp_cookie", encoded_state));

            let mut headers = HashMap::new();
            headers.insert("location".to_string(), location.to_string());
            (
                updated_jar,
                TmsHttpResponse::from((StatusCode::TEMPORARY_REDIRECT, headers)),
            )
        }

        Err(err) => {
            let error = TmsApiErrorResult {
                message: format!("{}", err),
            };

            (
                jar,
                TmsHttpResponse::from((StatusCode::BAD_REQUEST, Entity::from(error))),
            )
        }
    }
}
