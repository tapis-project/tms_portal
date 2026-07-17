use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::distr::Alphanumeric;
use rand::RngExt;
use rand::rngs::ThreadRng;
use sqlx::PgPool;
use url::Url;
use tms_lib::utils::service_error::{ServiceError::BadRequest};
use crate::db::allowed_redirects_dao::{db_get_allowed_redirect};
use crate::db::auth_code_data::db_insert_auth_code_data;
use crate::db::client_dao::db_get_client_by_id;
use crate::models::app_error::AppError;

// Temporary internal storage mockup for verification
struct AuthCodeData {
    client_id: String,
    user_id: String,
    redirect_uri: String,
}

type AuthCodeStorage = Arc<Mutex<HashMap<String, AuthCodeData>>>;
pub async fn handle_authorize_code_response(pool:&PgPool, client_id:&String,
                                            redirect_uri:&String, state:&Option<String>,
                                            scope:&Option<String>) -> anyhow::Result<Url, AppError> {
    // validate client id
    let mut tx = pool.begin().await?;
    let client = db_get_client_by_id(&mut tx, &client_id).await?;

    // validate scope -- must not be present for now
    if let Some(requested_scope) = scope {
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