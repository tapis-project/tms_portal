use crate::services::service_error::ServiceError::BadRequest;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{query, Error, PgTransaction, Row};

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AllowedRedirect {
    pub uri: String,
    pub client_id: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl From<&PgRow> for AllowedRedirect {
    fn from(row: &PgRow) -> Self {
        AllowedRedirect {
            uri: row.get("uri"),
            client_id: row.get("client_id"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn db_get_allowed_redirect<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: &String,
    redirect_uri: &String,
) -> Result<AllowedRedirect> {
    match query(
        "SELECT uri, client_id, created, updated FROM allowed_redirects WHERE client_id = $1 and uri = $2",
    )
        .bind(client_id)
        .bind(redirect_uri)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AllowedRedirect::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest("Invalid redirect uri".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
