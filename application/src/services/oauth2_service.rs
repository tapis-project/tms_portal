use std::collections::HashMap;
use std::time::SystemTime;
use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{DateTime, TimeDelta, Utc};
use rand::distr::Alphanumeric;
use rand::RngExt;
use rand::rngs::ThreadRng;
use serde_json::Value;
use sqlx::PgPool;
use url::Url;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::{ServiceError::BadRequest};
use tms_lib::utils::service_error::ServiceError::{Internal, Unauthorized};
use crate::db::allowed_redirects_dao::{db_get_allowed_redirect};
use crate::db::auth_code_data::{db_get_auth_code_data, db_insert_auth_code_data};
use crate::db::client_dao::{db_get_client_by_credentials, db_get_client_by_id, Client};
use crate::db::config_dao::{db_get_http_config, db_get_oauth_config};
use crate::db::identity_provider_dao::{db_get_login_provider_by_id, IdentityProvider};
use crate::models::app_error::AppError;
use crate::utils::jwt_utils::make_auth_token;
use crate::utils::state_utils::{decode_state, encode_state};
use crate::utils::oauth2_authorization_code_utils::{decode_access_token, get_login_provider_token, OAuth2State};

// Temporary internal storage mockup for verification
struct AuthCodeData {
    client_id: String,
    user_id: String,
    redirect_uri: String,
}
pub async fn authorize_code_response(pool:&PgPool, client_id:&String,
                                     redirect_uri:&String, state:&Option<String>,
                                     scope:&Option<String>) -> anyhow::Result<Url, AppError> {
    // validate client id
    let mut tx = pool.begin().await?;
    let client = db_get_client_by_id(&mut tx, &client_id).await?;

    // validate scope -- must not be present for now
    if let Some(_requested_scope) = scope {
        return Err(BadRequest(String::from("Scopes are not supported at present, and must not be requested.")).into());
    }

    // validate redirect uri
    let mut location_url = Url::parse(&redirect_uri)?;
    if let Some(fragment) = location_url.fragment() {
        if !fragment.is_empty() {
            return Err(BadRequest(format!("Redirect URI fragment is not allowed, but found {}", fragment)).into());
        }
    }
    let allowed_redirect =
        db_get_allowed_redirect(&mut tx, &client.id, &String::from(location_url.as_str())).await?;

    let auth_code = generate_code();
    db_insert_auth_code_data(&mut tx, &auth_code, &client.id, &allowed_redirect.uri).await?;

    location_url.query_pairs_mut()
        .append_pair("code", auth_code.as_str());

    if let Some(state_param) = state {
        location_url.query_pairs_mut()
            .append_pair("state", &state_param);
    }
    tx.commit().await?;
    Ok(location_url)
}

fn generate_code() -> String {
    ThreadRng::default().sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
pub async fn generate_code_and_redirect(pool:&PgPool, state:&OAuth2State) -> anyhow::Result<Url, AppError> {
    let auth_code = generate_code();
    let mut tx = pool.begin().await?;
    db_insert_auth_code_data(&mut tx, &auth_code, &state.client_id, &state.redirect_uri).await?;
    let auth_code_data = db_get_auth_code_data(
        &mut tx, &auth_code, &state.client_id, TimeDelta::seconds(20)).await?;
    tx.commit().await?;
    let mut location = Url::parse(&auth_code_data.redirect_uri)?;
    if let Some(state) = &state.client_state {
        location.query_pairs_mut().append_pair("state", &state);
    }
    location.query_pairs_mut().append_pair("code", &auth_code);
    Ok(location)
}

pub async fn get_login_redirect_location(pool:&PgPool, client_id:&String, redirect_uri:&String, encoded_state:&String) -> anyhow::Result<Url, AppError> {
    let mut tx = pool.begin().await?;
    let oauth_config = db_get_oauth_config(&mut tx).await?;
    let idp_id = oauth_config.login_oauth_provider;
    let idp = db_get_login_provider_by_id(&mut tx, &idp_id).await?;
    let http_config = db_get_http_config(&mut tx).await?;

    // Check redirect uri - this fails if the redirect doesnt exist
    db_get_allowed_redirect(&mut tx, &client_id, &redirect_uri).await?;

    tx.commit().await?;

    // need to get / ve

    // let oauth_state = OAuth2State {
    //     tms_identity: String::default(), // we don't have a tms identity at this point
    //     client_id,
    //     idp_id: idp_id.clone(),
    //     redirect_uri: redirect_uri.clone(),
    //     exp: SystemTime::now()
    //         .duration_since(SystemTime::UNIX_EPOCH)?
    //         .as_secs()
    //         // TODO:  This should be a config setting
    //         + 300000,
    // };
    //
    // let mut tx = pool.begin().await?;
    // let encoded_state = match encode_state(pool, oauth_state).await {
    //     Ok(state_string) => state_string,
    //     Err(error) => return Err(Internal(error.to_string()).into()),
    // };
    // tx.commit().await?;

    let callback_url = &http_config.get_oauth_provider_callback_url();
    let encoded_nonce = BASE64_STANDARD.encode(generate_nonce().to_ne_bytes());
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", &idp.client_id),
        ("redirect_uri", callback_url),
        ("state", &encoded_state),
        ("nonce", &encoded_nonce),
        ("access_type", "offline"),
    ];

    if let Some(scope) = &idp.scope {
        query_params.push(("scope", scope.as_str()))
    }

    Ok(Url::parse_with_params(&idp.identity_redirect_url, query_params)?)
}

// {
//     let updated_jar = jar.add(
//         Cookie::build((STATE_COOKIE_NAME, encoded_state))
//             .path(ROOT_COOKIE_PATH)
//             .http_only(true),
//     );
//
//     let mut headers = HashMap::new();
//     headers.insert("location".to_string(), location.to_string());
//     let creds = format!("{}:{}", idp.client_id, idp.client_secret);
//     let authorization = format!("Basic {}", BASE64_STANDARD.encode(&creds));
//     headers.insert("Authorization".to_string(), authorization);
//
//     Ok((
//         updated_jar,
//         TmsResponse::builder(StatusCode::TEMPORARY_REDIRECT)
//             .headers(headers)
//             .build(),
//     ))
//
// }
pub async fn get_login_identity_provider(pool:&PgPool) -> anyhow::Result<IdentityProvider, AppError> {
    let mut tx = pool.begin().await?;
    let oauth_config = db_get_oauth_config(&mut tx).await?;
    let idp_id = oauth_config.login_oauth_provider;
    let idp = db_get_login_provider_by_id(&mut tx, &idp_id).await?;
    tx.commit().await?;
    Ok(idp)
}

pub async fn get_client(pool:&PgPool, client_id:&String) -> anyhow::Result<Client, AppError> {
    let mut tx = pool.begin().await?;
    Ok(db_get_client_by_id(&mut tx, client_id).await?)
}

pub async fn get_state(pool:&PgPool, client_id:&String, redirect_uri:&String, idp_id:&String,
                       state:&Option<String>) -> anyhow::Result<String, AppError> {
    let mut tx = pool.begin().await?;
    // Check redirect uri - this fails if the redirect doesnt exist
    db_get_allowed_redirect(&mut tx, &client_id, &redirect_uri).await?;

    tx.commit().await?;

    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id: client_id.clone(),
        idp_id: idp_id.clone(),
        redirect_uri: redirect_uri.clone(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            // TODO:  This should be a config setting
            + 300000,
        nonce: generate_nonce(),
        client_state: state.clone(),
    };

    match encode_state(pool, oauth_state).await.context("Unable to encode state") {
        Ok(state_string) => Ok(state_string),
        Err(error) => Err(Internal(error.to_string()).into()),
    }
}

pub async fn token_from_code(pool:&PgPool, client_id:&String, client_secret:&String, code:&String,
                             redirect_uri:&String) -> anyhow::Result<String, AppError> {
    // validate client id
    let mut tx = pool.begin().await?;
    let client = db_get_client_by_credentials(&mut tx, &client_id, client_secret).await?;

    // validate redirect uri
    let allowed_redirect =
        db_get_allowed_redirect(&mut tx, &client.id, &redirect_uri).await?;

    tx.commit().await?;
    Err(Internal(String::from("not implemented yet")).into())
}

pub async fn do_authorize_callback(
    pool: &PgPool,
    state: &String,
    code: &String,
    cookie_state: &String,
) -> anyhow::Result<String> {
    if !cookie_state.eq(state) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state = decode_state(pool, state)
        .await
        .context("Unable to decode state query param")?;
    dbg!(&decoded_state);

    let mut tx = pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;

    let token = get_login_provider_token(pool, &idp, code, &http_config.get_oauth_provider_callback_url()).await?;
    dbg!(&token);

    let mut claims: HashMap<String, Value> = decode_access_token(&idp, &token.id_token).await?;
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    dbg!(&claims);

    let audience = String::from("FIXME!!");
    make_auth_token(pool, &decoded_state.client_id, &idp, claims).await
}

