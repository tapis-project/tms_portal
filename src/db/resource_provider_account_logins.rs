use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{query, Error, PgTransaction, Row};
use sqlx::postgres::PgRow;
use crate::db::allowed_redirects_dao::AllowedRedirect;
use crate::services::service_error::ServiceError::BadRequest;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLogin {
    pub id:i32,
    pub tms_user_id:String,
    pub resource_provider_account:String,
    pub resource_provider_id:String,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub auth_token:String,
    pub auth_token_expiration:DateTime<Utc>,
    pub refresh_token:String,
    pub refresh_token_expiration:DateTime<Utc>,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}

impl From<&PgRow> for ResourceAccountLogin {
    fn from(row: &PgRow) -> Self {
        ResourceAccountLogin {
            id:row.get("id"),
            tms_user_id:row.get("tms_user_id"),
            resource_provider_account:row.get("resource_provider_account"),
            resource_provider_id:row.get("resource_provider_id"),
            last_login:row.get("last_login"),
            enabled:row.get("enabled"),
            auth_token:row.get("auth_token"),
            auth_token_expiration:row.get("auth_token_expiration"),
            refresh_token:row.get("refresh_token"),
            refresh_token_expiration:row.get("refresh_token_expiration"),
            created:row.get("created"),
            updated:row.get("updated"),
        }
    }
}

pub async fn db_add_or_update_resource_account_login<'a>(
    tx: &mut PgTransaction<'a>, tms_user_id: String, resource_provider_account: String, resource_provider_id: String,
    last_login: DateTime<Utc>, enabled:bool, auth_token:String, auth_token_expiration:DateTime<Utc>,
    refresh_token:String, refresh_token_expiration:DateTime<Utc>,
) -> anyhow::Result<ResourceAccountLogin> {
    let raLogin = ResourceAccountLogin {
        id:0,
        tms_user_id,
        resource_provider_account,
        resource_provider_id,
        last_login,
        enabled,
        auth_token,
        auth_token_expiration,
        refresh_token,
        refresh_token_expiration,
        created:Utc::now(),
        updated:Utc::now(),
    };

    match query(
        "INSERT INTO resource_provider_account_logins
            (tms_user_id, resource_provider_account, resource_provider_id,
             enabled, auth_token, auth_token_expiration, refresh_token, refresh_token_expiration)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) returning *"
    )
    .bind(raLogin.tms_user_id)
    .bind(raLogin.resource_provider_account)
    .bind(raLogin.resource_provider_id)
    .bind(raLogin.enabled)
    .bind(raLogin.auth_token)
    .bind(raLogin.auth_token_expiration)
    .bind(raLogin.refresh_token)
    .bind(raLogin.refresh_token_expiration)
    .fetch_one(&mut **tx)
    .await {
        Ok(row) => Ok(ResourceAccountLogin::from(&row)),
//        Err(Error::RowNotFound) => Err(BadRequest("Invalid redirect uri".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
