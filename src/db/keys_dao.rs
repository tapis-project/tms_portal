use crate::services::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Key {
    pub kid: String,
    pub jwt_public_key: String,
    pub jwt_private_key: String,
    pub created: PrimitiveDateTime,
    pub updated: PrimitiveDateTime,
}

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
        "select kid, jwt_public_key, jwt_private_key, created, updated from keys where kid = $1",
    )
    .bind(kid)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Key kid {} not found", kid)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    dbg!(&row);
    Ok(Key::from(&row))
}
