use crate::db::client_dao::db_get_client_by_id;
use crate::db::config_dao::{db_get_http_config, db_get_jwt_config};
use crate::db::keys_dao::db_get_key_by_id;
use crate::models::tms_response::TmsResponse;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{debug_handler, Router};
use jwtk::jwk::JwkSet;
use jwtk::rsa::RsaPublicKey;
use jwtk::PublicKeyToJwk;
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::models::app_error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenidConfiguration {
    issuer: String,                 // "issuer": "https://auth.globus.org",
    authorization_endpoint: String, // "authorization_endpoint": "https://auth.globus.org/v2/oauth2/authorize",
    token_endpoint: String,         //"token_endpoint": "https://auth.globus.org/v2/oauth2/token",
    revocation_endpoint: String, // "revocation_endpoint": "https://auth.globus.org/v2/oauth2/token/revoke",
    jwks_uri: String,            // "jwks_uri": "https://auth.globus.org/jwk.json",
}
// more stuff we might add one day
// "response_types_supported": [
// "code",
// "token",
//     "token id_token",
//     "id_token"
//     ],
//     "id_token_signing_alg_values_supported": [
//     "RS512"
//     ],
// "scopes_supported": [
// "openid",
// "email",
// "profile"
// ],
// "token_endpoint_auth_methods_supported": [
// "client_secret_basic"
// ],
// "claims_supported": [
//     "at_hash",
//     "aud",
//     "email",
//     "exp",
//     "name",
//     "nonce",
//     "preferred_username",
//     "iat",
//     "iss",
//     "sub"
//     ],
//     "subject_types_supported" : ["public"]
// }

pub async fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/{client_id}/.well-known/openid-configuration",
            get(openid_configuration_handler),
        )
        .route("/{client_id}/jwks.json", get(jwks_json_handler))
}

#[debug_handler]
pub async fn openid_configuration_handler(
    Path(client_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<TmsResponse<OpenidConfiguration>, AppError> {
    let mut tx = app_state.db_pool.begin().await?;
    let http_config = db_get_http_config(&mut tx).await?;
    let openid_configuration = OpenidConfiguration {
        issuer: http_config.base_url.clone(),
        authorization_endpoint: http_config.get_authorization_url().clone(),
        token_endpoint: http_config.get_token_url().clone(),
        revocation_endpoint: http_config.get_revocation_url().clone(),
        jwks_uri: http_config.get_jwks_url().to_string(),
    };

    debug!("Client Id:{:?}", client_id);
    Ok(TmsResponse::builder(StatusCode::OK)
        .entity(openid_configuration)
        .build())
}
pub async fn jwks_json_handler(
    Path(client_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<TmsResponse<JwkSet>, AppError> {
    let mut tx = app_state.db_pool.begin().await?;
    // lookup client ot make sure it's a real client id
    let _client = db_get_client_by_id(&mut tx, &client_id).await?;
    let jwt_config = db_get_jwt_config(&mut tx).await?;
    let key = db_get_key_by_id(&mut tx, &jwt_config.signing_key_kid).await?;
    tx.commit().await?;

    let public_key_string = key.jwt_public_key;
    let rpk = RsaPublicKey::from_pem(public_key_string.as_bytes(), None)?;
    // let encoding_key = EncodingKey::from_rsa_pem(&public_key_string.as_bytes())?;
    // let jwk = Jwk::from_encoding_key(&encoding_key, RS256)?;
    // let jwks = JwkSet { keys: vec![jwk] };

    let jwk = rpk.public_key_to_jwk()?;
    let keys = vec![jwk];
    let jwks = JwkSet { keys };
    // let key = serde_json::to_string(&jwks)?;
    // let key = "stuff".to_string();
    Ok(TmsResponse::builder(StatusCode::OK).entity(jwks).build())
}
