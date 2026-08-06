use log::trace;
use tms_lib::utils::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::keys::Key;

impl From<&PgRow> for Key {
    fn from(row: &PgRow) -> Self {
        Key {
            kid: row.get("kid"),
            jwt_public_key: row.get("jwt_public_key"),
            jwt_private_key: row.get("jwt_private_key"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn db_get_key_by_id<'a>(tx: &mut PgTransaction<'a>, kid: &String) -> anyhow::Result<Key> {
    let row = query(
        "select * from keys where kid = $1",
    )
    .bind(kid)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Key kid {} not found", kid)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    Ok(Key::from(&row))
}
