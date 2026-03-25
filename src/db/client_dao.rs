use crate::services::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};

#[derive(Debug)]
pub struct Client {
    pub id: String,
    pub secret: String,
    pub name: String,
    pub created: PrimitiveDateTime,
    pub updated: PrimitiveDateTime,
    pub jwt_private_key: String,
    pub jwt_public_key: String,
}

impl From<&PgRow> for Client {
    fn from(row: &PgRow) -> Self {
        Client {
            id: row.get("id"),
            secret: row.get("secret"),
            name: row.get("name"),
            created: row.get("created"),
            updated: row.get("updated"),
            jwt_public_key: row.get("jwt_public_key"),
            jwt_private_key: row.get("jwt_private_key"),
        }
    }
}
pub async fn db_get_client_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
) -> anyhow::Result<Client> {
    let row = query("select id, name, secret, jwt_public_key, jwt_private_key, created, updated from clients where id = $1")
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Client id {} not found", id)).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(Client::from(&row))
}

pub async fn db_get_client_by_credentials<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
    secret: &String,
) -> anyhow::Result<Client> {
    let row = query("select id, name, secret, jwt_public_key, jwt_private_key, created, updated from clients where id = $1 and secret = $2")
        .bind(id)
        .bind(secret)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Client id {} not found", id)).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(Client::from(&row))
}
