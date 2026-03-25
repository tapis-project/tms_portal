use crate::db::client_dao::db_get_client_by_credentials;
use crate::services::service_error::ServiceError;
use crate::services::service_error::ServiceError::NotFound;
use anyhow::Error;
use sqlx::PgPool;

pub async fn basic_auth_is_authorized(
    pool: &PgPool,
    id: &String,
    secret: &String,
) -> Result<bool, Error> {
    let mut tx = pool.begin().await?;
    let result = db_get_client_by_credentials(&mut tx, &id, &secret).await;
    tx.commit().await?;

    match result {
        Ok(_) => Ok(true),
        Err(err) => {
            if let Some(error) = err.downcast_ref::<ServiceError>() {
                match error {
                    NotFound(_) => Ok(false),
                    _ => Err(err),
                }
            } else {
                Err(err)
            }
        }
    }
}
