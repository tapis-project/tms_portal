use std::collections::HashSet;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{query, Error, PgTransaction, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;
use tms_lib::utils::service_error::ServiceError::BadRequest;
use crate::obj_model::resources::{ResourceAccountLink, ResourceProviderLogin};

impl From<&PgRow> for ResourceProviderLogin {
    fn from(row: &PgRow) -> Self {
        ResourceProviderLogin {
            id:row.get("id"),
            tms_identity:row.get("tms_identity"),
            provider_account:row.get("provider_account"),
            provider_uuid:row.get("provider_uuid"),
            last_login:row.get("last_login"),
            enabled:row.get("enabled"),
            created:row.get("created"),
            updated:row.get("updated"),
        }
    }
}
impl From<&PgRow> for ResourceAccountLink {
    fn from(row: &PgRow) -> Self {
        ResourceAccountLink {
            id:row.get("id"),
            tms_identity:row.get("tms_identity"),
            resource_provider_account:row.get("provider_account"),
            resource_provider_uuid:row.get("provider_uuid"),
            resource_provider_id:row.get("provider_id"),
            resource_provider_name:row.get("provider_name"),
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
) -> anyhow::Result<ResourceProviderLogin> {
    let ra_login = ResourceProviderLogin {
        id:0,
        tms_identity: tms_identity,
        provider_account: resource_provider_account,
        provider_uuid: resource_provider_uuid,
        last_login,
        enabled,
        created:Utc::now(),
        updated:Utc::now(),
    };

    match query(
        "INSERT INTO resource_provider_logins
            (tms_identity, provider_account, provider_uuid,
             enabled, last_login) VALUES ($1, $2, $3, $4, $5)
                      ON CONFLICT (tms_identity, provider_uuid, provider_account)
                          DO UPDATE SET last_login=excluded.last_login
             returning *",
    )
    .bind(ra_login.tms_identity)
    .bind(ra_login.provider_account)
    .bind(ra_login.provider_uuid)
    .bind(ra_login.enabled)
    .bind(ra_login.last_login)
    .bind(Utc::now())
    .fetch_one(&mut **tx)
    .await {
        Ok(row) => Ok(ResourceProviderLogin::from(&row)),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_delete_resource_provider_link<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: &String, resource_provider_link_id: &i64
) -> anyhow::Result<ResourceProviderLogin> {
    match query(
        "delete from resource_provider_logins where
                     tms_identity = $1 and id = $2 returning *",
    ).bind(tms_identity)
        .bind(resource_provider_link_id)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(ResourceProviderLogin::from(&row)),
        Err(Error::RowNotFound) => Err(BadRequest(format!("Resource provider link id {} not found for tms_identity {}", resource_provider_link_id, tms_identity)).into()),
        Err(error) => Err(anyhow!(error))
    }
}
pub async fn db_get_resource_provider_links_for_identity<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: &String
) -> anyhow::Result<HashSet<ResourceAccountLink>> {
    let row_result = match query(
        // this select needs all of the fields spelled out because of the join and naming, etc
       "select rpal.id, rpal.tms_identity, rpal.provider_account, rpal.last_login, rpal.enabled,
       rpal.created, rpal.updated, ip.id as provider_id, ip.name as provider_name,
       ip.uuid as provider_uuid from resource_provider_logins as rpal
       INNER JOIN identity_providers AS ip ON ip.uuid = rpal.provider_uuid
       WHERE tms_identity=$1 AND ip.supports_resources = true",
    ).bind(tms_identity)
        .fetch_all(&mut **tx)
        .await {
        Ok(rows) => Ok(rows),
        Err(error) => Err(anyhow!(error))
    };

    let mut account_logins: Vec<ResourceAccountLink> = vec![];
    for row in &row_result? {
        account_logins.push(ResourceAccountLink::from(row));
    }

    Ok(HashSet::from_iter(account_logins))
}
