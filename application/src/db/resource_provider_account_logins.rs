use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{query, PgTransaction, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLogin {
    pub id:i32,
    pub tms_identity:String,
    pub resource_provider_account:String,
    pub resource_provider_uuid:Option<Uuid>,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}

impl From<&PgRow> for ResourceAccountLogin {
    fn from(row: &PgRow) -> Self {
        ResourceAccountLogin {
            id:row.get("id"),
            tms_identity:row.get("tms_identity"),
            resource_provider_account:row.get("resource_provider_account"),
            resource_provider_uuid:row.get("resource_provider_uuid"),
            last_login:row.get("last_login"),
            enabled:row.get("enabled"),
            created:row.get("created"),
            updated:row.get("updated"),
        }
    }
}

pub async fn db_add_or_update_resource_account_login<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: String, resource_provider_account: String,
    resource_provider_uuid: Option<Uuid>, last_login: DateTime<Utc>, enabled:bool,
) -> anyhow::Result<ResourceAccountLogin> {
    let ra_login = ResourceAccountLogin {
        id:0,
        tms_identity: tms_identity,
        resource_provider_account,
        resource_provider_uuid,
        last_login,
        enabled,
        created:Utc::now(),
        updated:Utc::now(),
    };

    match query(
        "INSERT INTO resource_provider_account_logins
            (tms_identity, resource_provider_account, resource_provider_uuid,
             enabled, last_login) VALUES ($1, $2, $3, $4, $5)
                      ON CONFLICT (tms_identity, resource_provider_uuid, resource_provider_account)
                          DO UPDATE SET last_login=excluded.last_login
             returning *",
    )
    .bind(ra_login.tms_identity)
    .bind(ra_login.resource_provider_account)
    .bind(ra_login.resource_provider_uuid)
    .bind(ra_login.enabled)
    .bind(ra_login.last_login)
    .bind(Utc::now())
    .fetch_one(&mut **tx)
    .await {
        Ok(row) => Ok(ResourceAccountLogin::from(&row)),
        Err(error) => Err(anyhow!(error)),
    }
}
