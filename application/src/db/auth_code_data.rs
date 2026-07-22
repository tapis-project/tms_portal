use anyhow::anyhow;
use chrono::{DateTime, TimeDelta, Utc};
use sqlx::postgres::PgRow;
use sqlx::{query, Error, PgTransaction, Row};
use tms_lib::utils::service_error::ServiceError::BadRequest;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AuthCodeData {
    pub auth_code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl From<&PgRow> for AuthCodeData {
    fn from(row: &PgRow) -> Self {
        AuthCodeData {
            auth_code: row.get("auth_code"),
            client_id: row.get("client_id"),
            redirect_uri: row.get("redirect_uri"),
            state: row.get("state"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn db_get_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    expiration: TimeDelta,
) -> anyhow::Result<AuthCodeData> {
    let current_time = Utc::now();
    let earliest_good_date_time = current_time - expiration;
    match query(
        "SELECT * FROM auth_code_data WHERE auth_code = $1 AND client_id = $2 AND created > $3",
    )
        .bind(auth_code)
        .bind(client_id)
        .bind(earliest_good_date_time)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest("Auth code data not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_insert_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
) -> anyhow::Result<AuthCodeData> {
    match query(
        "insert into auth_code_data (auth_code, client_id, redirect_uri) VALUES ($1, $2,$3) returning *",
    )
        .bind(auth_code)
        .bind(client_id)
        .bind(redirect_uri)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::from(&row)),
        Err(error) => Err(anyhow!(error)),
    }
}
