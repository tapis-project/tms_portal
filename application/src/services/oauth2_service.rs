use std::time::SystemTime;
use anyhow::Context;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use chrono::{TimeDelta};
use rand::distr::Alphanumeric;
use rand::RngExt;
use rand::rngs::ThreadRng;
use serde_json::{from_value, Value};
use sqlx::{PgPool};
use url::Url;
use tms_lib::utils::oauth_utils::generate_nonce;
use tms_lib::utils::service_error::ServiceError::{Internal, Unauthorized};
use crate::db::allowed_redirects_dao::{db_get_allowed_redirect};
use crate::db::auth_code_data_dao::{db_delete_auth_code_data, db_insert_auth_code_data};
use crate::db::client_dao::{db_get_client_by_credentials,};
use crate::db::config_dao::{db_get_http_config};
use crate::db::identity_provider_dao::{db_get_login_provider_by_id};
use crate::db::issued_tokens_dao::db_insert_token;
use crate::obj_model::identity_provider::IdentityProvider;
use crate::services::login_service::AuthorizationCodeResponse;
use crate::utils::configuration::Configuration;
use crate::utils::jwt_utils::{get_tms_token_claims, make_auth_token, JwtClaims};
use crate::utils::state_utils::{decode_state, encode_state};
use crate::utils::oauth2_authorization_code_utils::{decode_access_token, get_token_for_provider, OAuth2State};

pub struct AuthorizationResult {
    pub location: String,
    pub encoded_state: String,
}

pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: String,
    pub expires_in: i64,
    pub id_token: String,
    pub jti: String,
}

pub struct AuthorizationCallbackResult {
    pub location: String,
}

fn generate_code() -> String {
    ThreadRng::default().sample_iter(Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub async fn authorize_code(db_pool:&PgPool, state:&Option<String>, client_id:&String, redirect_uri:&String) -> anyhow::Result<AuthorizationResult> {
    // get what we need from the database
    let mut tx = db_pool.begin().await?;

    let configuration = Configuration::get(&db_pool).await?;
    let idp = db_get_login_provider_by_id(&mut tx, &configuration.oauth_config.login_oauth_provider).await?;

    // Check redirect uri - this fails if the redirect doesnt exist
    let _allowed_redirect = db_get_allowed_redirect(&mut tx, &client_id, &redirect_uri).await?;
    tx.commit().await?;

    // encode our state
    let encoded_state = get_state(&db_pool, &client_id,
                                  redirect_uri, &idp.id, state).await?;

    // compute the location url that will be used for the redirect
    let location = get_login_redirect_location(&configuration, &idp, &encoded_state).await?;

    Ok(AuthorizationResult {
        location: location.to_string(),
        encoded_state
    })
}

pub async fn generate_code_and_redirect(pool:&PgPool, configuration:&Configuration, identity_provider: &IdentityProvider, state:&OAuth2State, provider_token:&String) -> anyhow::Result<Url> {
    let auth_code = generate_code();

    let mut claims: JwtClaims = decode_access_token(&identity_provider, &provider_token).await?;
    // TODO: get iss from config
    claims.insert("iss".to_string(), Value::from("https://tms.tacc.edu/"));
    let tms_token_claims = get_tms_token_claims(configuration, &state.client_id, &identity_provider.id, &identity_provider.identity_provider_type, &claims).await?;

    let mut tx = pool.begin().await?;
    db_insert_auth_code_data(&mut tx, &auth_code, &state.client_id, &state.redirect_uri,
                             &tms_token_claims, &identity_provider.id, &identity_provider.identity_provider_type).await?;
    tx.commit().await?;

    let mut location = Url::parse(&state.redirect_uri)?;
    if let Some(state) = &state.client_state {
        location.query_pairs_mut().append_pair("state", &state);
    }
    location.query_pairs_mut().append_pair("code", &auth_code);
    Ok(location)
}

async fn get_login_redirect_location(configuration:&Configuration,
                                         identity_provider: &IdentityProvider,
                                         encoded_state:&String) -> anyhow::Result<Url> {
    let callback_url = &configuration.http_config.get_oauth_provider_callback_url();
    let encoded_nonce = BASE64_STANDARD.encode(generate_nonce().to_ne_bytes());
    let mut query_params = vec![
        ("response_type", "code"),
        ("client_id", &identity_provider.client_id),
        ("redirect_uri", callback_url),
        ("state", &encoded_state),
        ("nonce", &encoded_nonce),
        ("access_type", "offline"),
    ];

    if let Some(scope) = &identity_provider.scope {
        query_params.push(("scope", scope.as_str()))
    }

    Ok(Url::parse_with_params(&identity_provider.identity_redirect_url, query_params)?)
}
async fn get_state(pool:&PgPool, client_id:&String, redirect_uri:&String, idp_id:&String,
                       state:&Option<String>) -> anyhow::Result<String> {

    // populate state structure
    let oauth_state = OAuth2State {
        tms_identity: String::default(), // we don't have a tms identity at this point
        client_id: client_id.clone(),
        idp_id: idp_id.clone(),
        redirect_uri: redirect_uri.clone(),
        exp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            // TODO:  This should be a config setting
            + 300,
        nonce: generate_nonce(),
        client_state: state.clone(),
    };

    // encode state structure
    match encode_state(pool, oauth_state).await.context("Unable to encode state") {
        Ok(state_string) => Ok(state_string),
        Err(error) => Err(Internal(error.to_string()).into()),
    }
}

pub async fn get_access_token_from_code(db_pool:&PgPool, client_id:&String, client_secret:&String, code:&String,
                                        redirect_uri:&String) -> anyhow::Result<TokenResponse> {
    // validate client id
    let mut tx = db_pool.begin().await?;
    let client = db_get_client_by_credentials(&mut tx, &client_id, client_secret).await?;
    let configuration = Configuration::get(db_pool).await?;
    // validate redirect uri
    let _allowed_redirect =
        db_get_allowed_redirect(&mut tx, &client.id, &redirect_uri).await?;
    // TODO: get time delta fron config (how recently the auth code must have been issued)
    let time_delta = TimeDelta::seconds(30);

    // this will remove the auth code data, but return it.  It's only valid for a single use, so we
    // don't want it hanging around.
    // TODO: we could invalidate the record insted if we think it might help debuging / tracking errors down.  They would still need to be removed at some point, but we could do that once a day or whatever.  I prefer just deleting them I think.
    let auth_code_data = db_delete_auth_code_data(&mut tx, code, &client_id, &redirect_uri, time_delta).await?;
    let idp = db_get_login_provider_by_id(&mut tx, &auth_code_data.idp_id).await?;
    tx.commit().await?;

    let claims = from_value(auth_code_data.claims)?;
    let tms_token_claims = get_tms_token_claims(&configuration, &client_id, &idp.id, &idp.identity_provider_type, &claims).await?;
    let tms_token_string = make_auth_token(db_pool, &tms_token_claims).await?;

    Ok(TokenResponse{
        access_token: tms_token_string.clone(),
        expires_at: tms_token_claims.get_expires_at()?,
        expires_in: tms_token_claims.get_expires_in()?,
        id_token: String::from(""),
        jti: tms_token_claims.get_jti()?
    })
}

pub async fn get_provider_token(
    pool: &PgPool,
    state: &String,
    code: &String,
    cookie_state: &String,
) -> anyhow::Result<String> {
    if !cookie_state.eq(state) {
        return Err(Unauthorized("State cookies do not match".to_string()).into());
    }

    let decoded_state:OAuth2State = decode_state(pool, state)
        .await
        .context("Unable to decode state query param")?;

    let mut tx = pool.begin().await?;
    let idp = db_get_login_provider_by_id(&mut tx, &decoded_state.idp_id)
        .await
        .context("Unable to get idp for database")?;
    let http_config = db_get_http_config(&mut tx).await?;
    tx.commit().await?;

    let token:AuthorizationCodeResponse = get_token_for_provider(&idp, &http_config.get_oauth_provider_callback_url(), code).await?;
    Ok(token.id_token)
}

pub async fn process_authorization_callback(db_pool:&PgPool, internal_oauth_state:&String, client_state:&String, code:&String) -> anyhow::Result<AuthorizationCallbackResult> {
    // redirect browser back to the post-login page (taken from state - validated in login step).
    let decoded_internal_state:OAuth2State = decode_state(db_pool, internal_oauth_state).await?;

    let mut tx = db_pool.begin().await?;
    let configuration= Configuration::get(db_pool).await?;
    let identity_provider = db_get_login_provider_by_id(&mut tx, &decoded_internal_state.idp_id).await?;
    tx.commit().await?;

    // exchange code for token (state validated in handle_callback)
    let provider_token = get_provider_token(
        db_pool,
        client_state,
        code,
        internal_oauth_state,
    ).await?;


    let location = generate_code_and_redirect(db_pool, &configuration, &identity_provider, &decoded_internal_state, &provider_token).await?;
    Ok(AuthorizationCallbackResult{
        location: location.to_string(),
    })
}

