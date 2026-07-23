use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::PgPool;
use tms_lib::utils::jwt_decoder::JwtDecoderBuilder;
use tms_lib::utils::jwt_encoder::JwtEncoderBuilder;
use crate::db::config_dao::db_get_state_key_id;
use crate::db::keys_dao::db_get_key_by_id;
use crate::utils::oauth2_authorization_code_utils::OAuth2State;

const DEFAULT_STATE_ALGORITHM: &str = "RS256";

pub async fn encode_state<T>(pool: &PgPool, oauth_state: T) ->
            anyhow::Result<String> where T:Serialize {
    let mut tx = pool.begin().await?;
    let state_key = db_get_state_key_id(&mut tx).await?;
    let keys = db_get_key_by_id(&mut tx, &state_key.kid).await?;
    tx.commit().await?;

    JwtEncoderBuilder::builder(
        oauth_state,
        &keys.jwt_private_key.as_bytes(),
        DEFAULT_STATE_ALGORITHM,
        keys.kid.as_str(),
    )
        .encode()
        .await
}

pub async fn decode_state<T>(pool: &PgPool, state_string: &String) ->
            anyhow::Result<T> where T:DeserializeOwned {
    let mut tx = pool.begin().await?;
    let state_key = db_get_state_key_id(&mut tx).await?;
    let keys = db_get_key_by_id(&mut tx, &state_key.kid).await?;
    tx.commit().await?;
    // let decoding_key = Some(DecodingKey::from_rsa_pem(&keys.jwt_public_key.as_bytes())?);

    JwtDecoderBuilder::builder()
        .public_key(&keys.jwt_public_key.as_bytes())
        .validate_aud(false)
        .decode::<T>(&state_string)
        .await
}

