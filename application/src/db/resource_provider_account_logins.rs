use std::collections::HashSet;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{query, Error, PgTransaction, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use tms_lib::utils::service_error::ServiceError::BadRequest;

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
#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceAccountLinks {
    pub id:i32,
    pub tms_identity:String,
    pub resource_provider_account:String,
    pub resource_provider_uuid:Uuid,
    pub resource_provider_id:String,
    pub resource_provider_name:String,
    pub last_login:DateTime<Utc>,
    pub enabled:bool,
    pub created:DateTime<Utc>,
    pub updated:DateTime<Utc>,
}
impl From<&PgRow> for ResourceAccountLinks {
    fn from(row: &PgRow) -> Self {
        ResourceAccountLinks {
            id:row.get("id"),
            tms_identity:row.get("tms_identity"),
            resource_provider_account:row.get("resource_provider_account"),
            resource_provider_uuid:row.get("resource_provider_uuid"),
            resource_provider_id:row.get("resource_provider_id"),
            resource_provider_name:row.get("resource_provider_name"),
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
pub async fn db_delete_resource_provider_link<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: &String, resource_provider_link_id: &i64
) -> anyhow::Result<ResourceAccountLogin> {
    match query(
        "delete from resource_provider_account_logins where
                     tms_identity = $1 and id = $2 returning *",
    ).bind(tms_identity)
        .bind(resource_provider_link_id)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(ResourceAccountLogin::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest(format!("Resource provider link id {} not found for tms_identity {}", resource_provider_link_id, tms_identity)).into()),
        Err(error) => Err(anyhow!(error))
    }
}
pub async fn db_get_resource_provider_links_for_identity<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: &String
) -> anyhow::Result<HashSet<ResourceAccountLinks>> {
    let row_result = match query(
        // this select needs all of the fields spelled out because of the join and naming, etc
       "select rpal.id, rpal.tms_identity, rpal.resource_provider_account, rpal.last_login, rpal.enabled,
       rpal.created, rpal.updated, ip.id as resource_provider_id, ip.name as resource_provider_name,
       ip.uuid as resource_provider_uuid from resource_provider_account_logins as rpal
       INNER JOIN identity_providers AS ip ON ip.uuid = rpal.resource_provider_uuid
       WHERE tms_identity=$1 AND ip.supports_resources = true",
    ).bind(tms_identity)
        .fetch_all(&mut **tx)
        .await {
        Ok(rows) => Ok(rows),
        Err(error) => Err(anyhow!(error))
    };

    let mut account_logins: Vec<ResourceAccountLinks> = vec![];
    for row in &row_result? {
        account_logins.push(ResourceAccountLinks::from(row));
    }

    Ok(HashSet::from_iter(account_logins))
}
