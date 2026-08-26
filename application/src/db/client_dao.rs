use tms_lib::utils::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::client::Client;

impl From<&PgRow> for Client {
    fn from(row: &PgRow) -> Self {
        Client {
            id: row.get("id"),
            client_id: row.get("client_id"),
            secret: row.get("secret"),
            name: row.get("name"),
            enabled: row.get("enabled"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}
pub async fn db_get_client_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
) -> anyhow::Result<Client> {
    let row = query("select * from clients where id = $1")
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Client id {} not found", id)).into(),
            _ => anyhow::anyhow!(error),
        })?;
    Ok(Client::from(&row))
}

pub async fn db_get_client_by_credentials<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
    secret: &String,
) -> anyhow::Result<Client> {
    let row = query(
        "select * from clients where id = $1 and secret = $2",
    )
    .bind(id)
    .bind(secret)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Client id {} not found", id)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    Ok(Client::from(&row))
}
