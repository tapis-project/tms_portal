use std::collections::HashMap;
use anyhow::anyhow;
use chrono::{DateTime, TimeDelta, Utc};
use serde_json::{to_value, Map, Value};
use sqlx::postgres::PgRow;
use sqlx::{query, Error, PgTransaction, Row};
use tms_lib::utils::service_error::ServiceError::{BadRequest};
use crate::db::identity_provider_dao::{IdentityProviderType};
use crate::utils::jwt_utils::TmsTokenClaims;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct AuthCodeData {
    pub auth_code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub idp_id: String,
    pub idp_type: IdentityProviderType,
    pub claims: Value,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl TryFrom<&PgRow> for AuthCodeData {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("idp_type");
        Ok(AuthCodeData {
            auth_code: row.get("auth_code"),
            client_id: row.get("client_id"),
            idp_id: row.get("idp_id"),
            redirect_uri: row.get("redirect_uri"),
            idp_type: provider.parse::<IdentityProviderType>()?,
            claims: row.get("claims"),
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}

pub async fn db_get_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
    expiration: TimeDelta,
) -> anyhow::Result<AuthCodeData> {
    let current_time = Utc::now();
    let earliest_good_date_time = current_time - expiration;
    match query(
        "SELECT * FROM auth_code_data WHERE auth_code = $1 AND client_id = $2 AND redirect_uri = $3 AND updated > $4",
    )
        .bind(auth_code)
        .bind(client_id)
        .bind(redirect_uri)
        .bind(earliest_good_date_time)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::try_from(&row)?),
        Err(Error::RowNotFound) => Err(BadRequest("Auth code data not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_delete_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
    expiration: TimeDelta,
) -> anyhow::Result<AuthCodeData> {
    let current_time = Utc::now();
    let earliest_good_date_time = current_time - expiration;
    match query(
        "DELETE FROM auth_code_data WHERE auth_code = $1 AND client_id = $2 AND redirect_uri = $3 AND updated > $4 returning *",
    )
        .bind(auth_code)
        .bind(client_id)
        .bind(redirect_uri)
        .bind(earliest_good_date_time)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::try_from(&row)?),
        Err(Error::RowNotFound) => Err(BadRequest("Auth code data not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_insert_auth_code_data<'a>(
    tx: &mut PgTransaction<'a>,
    auth_code: &String,
    client_id: &String,
    redirect_uri: &String,
    claims: &TmsTokenClaims,
    idp_id: &String,
    idp_type: &IdentityProviderType,
) -> anyhow::Result<AuthCodeData> {
    let claims_json = to_value(claims)?;
    match query(
        "insert into auth_code_data (auth_code, client_id, redirect_uri, claims, idp_id, idp_type) VALUES ($1, $2, $3, $4, $5, $6) returning *",
    )
        .bind(auth_code)
        .bind(client_id)
        .bind(redirect_uri)
        .bind(claims_json)
        .bind(idp_id)
        .bind(idp_type.to_string())
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(AuthCodeData::try_from(&row)?),
        Err(error) => Err(anyhow!(error)),
    }
}
