use crate::db::client_dao::db_get_client_by_credentials;
use crate::services::service_error::ServiceError;
use crate::services::service_error::ServiceError::{NotFound, Unauthorized};
use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sqlx::PgPool;

pub async fn basic_auth_is_authorized(pool: &PgPool, id: &String, secret: &String) -> Result<bool> {
    let mut tx = pool.begin().await?;
    let result = db_get_client_by_credentials(&mut tx, &id, &secret).await;
    tx.commit().await?;

    match result {
        Ok(_) => Ok(true),
        Err(err) => {
            if let Some(error) = err.downcast_ref::<ServiceError>() {
                match error {
                    NotFound(_) => Err(Unauthorized("Unauthorized".to_string()).into()),
                    _ => Err(err),
                }
            } else {
                Err(err)
            }
        }
    }
}

pub async fn basic_auth_from_header_value(pool: &PgPool, auth_header_value: &str) -> Result<bool> {
    let auth_string = auth_header_value.strip_prefix("Basic ");
    if let Some(auth_string) = auth_string {
        let decoded_bytes = BASE64_STANDARD.decode(auth_string)?;
        let decoded = String::from_utf8(decoded_bytes)?;
        if let Some((id, secret)) = decoded.split_once(":") {
            let id_string = id.to_string();
            let secret_string = secret.to_string();
            return basic_auth_is_authorized(pool, &id_string, &secret_string).await;
        }
    }

    Err(Unauthorized("Authorization: Basic header value not found.".to_string()).into())
}
