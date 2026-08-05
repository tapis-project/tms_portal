use chrono::{DateTime, Utc};
use tms_lib::utils::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::oauth::IssuedToken;

impl From<&PgRow> for IssuedToken {
    fn from(row: &PgRow) -> Self {
        IssuedToken {
            access_token: row.get("access_token"),
            expiration: row.get("expiration"),
            revoked: row.get("revoked"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}
pub async fn db_get_token<'a>(
    tx: &mut PgTransaction<'a>,
    token: &String,
) -> anyhow::Result<IssuedToken> {
    let row = query("select * from issued_tokens where access_token = $1")
        .bind(token)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Token not found")).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(IssuedToken::from(&row))
}
pub async fn db_delete_token<'a>(
    tx: &mut PgTransaction<'a>,
    token: &String,
) -> anyhow::Result<IssuedToken> {
    let row = query("delete from issued_tokens where access_token = $1")
        .bind(token)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(IssuedToken::from(&row))
}

pub async fn db_revoke_token<'a>(
    tx: &mut PgTransaction<'a>,
    token: &String,
) -> anyhow::Result<IssuedToken> {
    let row = query("update issued_tokens set revoked = true where access_token = $1 returning *")
        .bind(token)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(IssuedToken::from(&row))
}

pub async fn db_insert_token<'a>(
    tx: &mut PgTransaction<'a>,
    token: &String,
    expiration: &DateTime<Utc>,
) -> anyhow::Result<IssuedToken> {
    let row = query("insert into issued_tokens (access_token, expiration, revoked) values ($1, $2, false) returning *")
        .bind(token)
        .bind(expiration)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound("Token not found".to_string()).into(),
            _ => anyhow::anyhow!(error),
        })?;
    dbg!(&row);
    Ok(IssuedToken::from(&row))
}
