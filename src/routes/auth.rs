use crate::db::client_dao::db_get_client_by_credentials;
use crate::db::idp_dao::db_get_idp_by_id;
use crate::models::api::{Entity, TmsResponse, TokenResponse};
use crate::models::oauth2::{AuthCodeQueryParams, AuthorizeByIdpRequest};
use crate::routes::auth;
use crate::services::oauth_service::OAuthState;
use crate::services::oauth_service::{encode_state, get_idps, handle_callback, IdpResponse};
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use crate::AppState;
use anyhow::Result;
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, IntoResponseParts, Response, ResponseParts};
use axum::routing::post;
use axum::{debug_handler, extract::Query, http, routing::get, Form, Router};
use axum_extra::extract::PrivateCookieJar;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use jsonwebtoken::Header;
use reqwest::StatusCode;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::time::SystemTime;
use url::Url;

const CLIENT_ID_COOKIE_PATH: &str = "tms/oauth2/client_id";
const STATE_COOKIE_PATH: &str = "tms/oauth2/state_cookie";
pub async fn router(state: &AppState) -> Router<AppState> {
    let secure = Router::new()
        .route("/oauth2/callback", get(callback_handler))
        .route("/oauth2/idp", get(get_idp_handler))
        .route("/oauth2/authorize", post(authorize_handler))
        .route("/oauth2/authorize", get(authorize_handler));
    //        .layer(middleware::from_fn_with_state(state.clone(), my_middleware));
    let open = Router::new().route("/oauth2/login", get(login_handler));
    Router::new().merge(secure).merge(open)
}

async fn my_middleware(
    // run the `HeaderMap` extractor
    //    headers: HeaderMap,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    //    State(state):State<AppState>,
    //   jar:PrivateCookieJar,
    State(app_state): State<AppState>,
    jar: PrivateCookieJar,
    request: Request,
    next: Next,
) -> Response {
    let auth_header_value = request
        .headers()
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let auth_string = auth_header_value
        .as_ref()
        .map(|value| value.as_str().strip_prefix("Basic ").unwrap_or(value));

    let mut tx = app_state.db_pool.begin().await.unwrap();
    if let Some(auth_string) = auth_string {
        println!("Found Auth: {0}", &auth_string);
        let decoded_bytes = BASE64_STANDARD.decode(auth_string).unwrap();
        let decoded = String::from_utf8(decoded_bytes).unwrap();
        let (id, secret) = decoded.split_once(':').unwrap();
        println!("Decoded Auth: {0}", &decoded,);
        println!("Decoded Auth: Id:{0} Secret:{1}", &id, &secret);
        let result =
            db_get_client_by_credentials(&mut tx, &id.to_owned(), &secret.to_owned()).await;
        match result {
            Ok(client) => {
                println!("Found client: {:?}", &client);
                //let pcj = PrivateCookieJar::from_headers(request.headers(), app_state.key);
                //                request.headers_mut().insert(http::header::SET_COOKIE, jar.into_response_parts())
                request.uri().query().insert(&format!("client_id={0}", id));
                let updated_jar = jar.add("tms/oauth2/client_id");
                return next.run(request).await;
            }
            Err(err) => return StatusCode::UNAUTHORIZED.into_response(),
        }
        //        let response = next.run(request).await;
    }

    // do something with `response`...

    (StatusCode::UNAUTHORIZED).into_response()
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
    jar: PrivateCookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(PrivateCookieJar, TmsResponse<()>), AppError> {
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

            let client_id = "tms_test_client_id";
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
#[debug_handler]
pub async fn login_handler() -> Result<Html<String>, AppError> {
    let value = read_to_string("./login.html")?;
    Ok(Html(value))
}
