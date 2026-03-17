use crate::db::config_dao::{
    get_client_id, get_client_secret, get_jwt_private_key, get_state_private_key,
    get_state_public_key,
};
use crate::db::idp_dao::Idp;
use crate::models::tms_internal::{OAuthState, TmsResult};
use crate::utils::jwt_utils::{JwtDecoderBuilder, JwtEncoderBuilder};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub async fn decode_state(state_string: &String) -> TmsResult<OAuthState> {
    let decoding_key = match DecodingKey::from_rsa_pem(&get_state_public_key().as_bytes()) {
        Ok(decoding_key) => decoding_key,
        Err(error) => return Err(error.to_string()),
    };

    JwtDecoderBuilder::builder()
        .public_key(decoding_key)
        .decode::<OAuthState>(&state_string)
        .await
}

pub async fn encode_state(oauth_state: OAuthState) -> TmsResult<String> {
    let header = Header::new(Algorithm::RS256);
    let key = match EncodingKey::from_rsa_pem(get_state_private_key().as_bytes()) {
        Ok(key) => key,
        Err(error) => return Err(error.to_string()),
    };

    JwtEncoderBuilder::builder(header, oauth_state, key)
        .encode()
        .await
}

pub async fn exchange_code_for_token<R>(idp: &Idp, code: &String) -> TmsResult<R>
where
    R: for<'a> Deserialize<'a>,
{
    let form_params = [
        ("grant_type", "authorization_code".to_string()),
        ("client_id", get_client_id()),
        ("client_secret", get_client_secret()),
        ("code", code.to_owned()),
    ];
    let client = reqwest::Client::new();
    let response = client
        .post(&idp.oauth2_token_url)
        .form(&form_params)
        .send()
        .await;

    match response {
        Ok(response) => {
            let token_string = match response.text().await {
                Ok(response_string) => response_string,
                Err(error) => return Err(format!("Error getting response body: {}", error)),
            };

            let token = match serde_json::from_str::<R>(&token_string) {
                Ok(token) => token,
                Err(error) => return Err(error.to_string()),
            };

            Ok(token)
        }
        Err(error) => Err(error.to_string()),
    }
}

pub async fn decode_access_token<T>(idp: &Idp, id_token: &String) -> TmsResult<T>
where
    T: for<'a> Deserialize<'a>,
{
    let audience = HashSet::from([get_client_id()]);
    match JwtDecoderBuilder::builder()
        .jwks_url(&idp.oauth2_jwks_url)
        .audience(audience)
        .decode(id_token)
        .await
    {
        Ok(decoded) => Ok(decoded),
        Err(error) => Err(format!("Error decoding JWT: {}", error)),
    }
}

pub async fn make_auth_token(claims: HashMap<String, Value>) -> TmsResult<String> {
    let header = Header::new(Algorithm::RS256);
    let encoding_key = match EncodingKey::from_rsa_pem(&get_jwt_private_key().into_bytes()) {
        Ok(key) => key,
        Err(error) => return Err(error.to_string()),
    };

    // TODO: add kid, alg, and jti in header ... maybe other stuff?

    JwtEncoderBuilder::builder(header, claims, encoding_key)
        .encode()
        .await
}
