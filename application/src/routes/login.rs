use crate::db::allowed_redirects_dao::db_get_allowed_redirect;
use crate::db::config_dao::db_get_http_config;
use crate::db::identity_provider_dao::db_get_login_provider_by_id;
use crate::models::tms_response::TmsResponse;
use crate::models::login_api::{AuthorizeByIdpRequest, IdentityProvider, WhoAmIResponse};
use crate::services::login_service::{
    decode_state, encode_state, get_identity_providers, handle_callback, whoami,
};
use tms_lib::utils::service_error::ServiceError::{BadRequest, Internal, Unauthorized};
use crate::utils::oauth2_authorization_code_utils::{
    AuthCodeQueryParams, OAuth2State, CLIENT_ID_TMS, ROOT_COOKIE_PATH, STATE_COOKIE_NAME,
    TOKEN_COOKIE_NAME,
};
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
use crate::models::app_error::AppError;
/*
This file handles the web part of logging into the TMS portal.  This includes tasks such as:
- getting the list of login identity providers
- requesting a login (which involves a redirect to the browser to login)
- handling the callback (i.e. redirect from the identity provider login)
- whoami - information about the logged in user extracted from the token
 */

pub async fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/login", get(login_handler))
        .route("/login/whoami", get(whoami_handler))
        .route("/login/callback", get(callback_handler))
        .route("/login/idps", get(get_idp_handler))
}

/*
Accepts a form with information about the login (idp and redirect uri).  There's an
associated table of redirect uris that are allowed.  The redirect url must be in that
table.  This tells us where to redirect back to after the login.  Having multiple allowed
redirects allows for easier debugging - redirect back to localhost if debugging, etc.
 */
#[debug_handler]
pub async fn login_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    form_data: Form<AuthorizeByIdpRequest>,
) -> Result<(CookieJar, TmsResponse<()>), AppError> {
    // Portal login will always be the tms client id
    let client_id = String::from(CLIENT_ID_TMS);
    let mut tx = app_state.db_pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &form_data.idp_id).await;

    // we will not use this value, but we need to make sure this redirect uri is in the database.
    let _ = db_get_allowed_redirect(&mut tx, &client_id, &form_data.redirect_uri).await?;
    tx.commit().await?;

    match idp {
        Ok(idp) => {
            let oauth_state = OAuth2State {
                tms_identity: String::default(), // we don't have a tms identity at this point
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
            let callback_url = &http_config.get_identity_provider_callback_url();
            let mut nonce = [0u8; 12];
            OsRng.fill_bytes(&mut nonce);
            let nonce_slice = BASE64_STANDARD.encode(nonce);
            // TODO:  make a real nonce
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

/*
This method requires the user to be logged in (token in Authorization: Bearer token).
Information can be returned back from the verified/valid token such as the user's name.
 */
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

/*
OAuth2 Authorization code callback.  This code accepts the code from the login idp,
and exchanges it for the token from the login idp.  A TMS token is created and
returned.
 */
#[debug_handler]
pub async fn callback_handler(
    State(app_state): State<AppState>,
    jar: CookieJar,
    query_params: Query<AuthCodeQueryParams>,
) -> anyhow::Result<(CookieJar, TmsResponse<()>), AppError> {
    // Get the state cookie set during the login process.
    let Some(state_cookie) = jar.get(STATE_COOKIE_NAME) else {
        return Err(Unauthorized("No state cookies were found".to_string()).into());
    };

    // exchange code for token (state validated in handle_callback)
    let token = handle_callback(
        &app_state.db_pool,
        &query_params.state,
        &query_params.code,
        &state_cookie.value().to_owned(),
    )
    .await?;

    // Build a new cookie and save it with the TMS token.
    let c = Cookie::build((TOKEN_COOKIE_NAME, token))
        .path(ROOT_COOKIE_PATH)
        .http_only(false)
        .secure(true)
        .build();
    let updated_jar = jar.clone().add(c);

    // redirect browser back to the post-login page (taken from state - validated in login step).
    let decoded_state = decode_state(&app_state.db_pool, &state_cookie.value().to_owned()).await?;
    let headers: HashMap<String, String> =
        HashMap::from_iter(vec![(LOCATION.to_string(), decoded_state.redirect_uri)].into_iter());

    let updated_jar = updated_jar.remove(Cookie::from(STATE_COOKIE_NAME));

    Ok((
        updated_jar,
        TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
            .headers(headers)
            .build(),
    ))
}

/*
Return a list of logon identity providers
 */
#[debug_handler]
pub async fn get_idp_handler(
    State(app_state): State<AppState>,
) -> anyhow::Result<TmsResponse<HashSet<IdentityProvider>>, AppError> {
    let idp_result = get_identity_providers(&app_state.db_pool).await?;

    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(idp_result)
        .build())
}
