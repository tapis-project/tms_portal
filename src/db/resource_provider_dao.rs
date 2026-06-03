use crate::services::service_error::ServiceError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct ResourceProvider {
    pub id: String,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub identity_redirect_url: String,
    pub oauth2_token_url: String,
    pub oauth2_jwks_url: Option<String>,
    pub oidc_user_info_url: Option<String>,
    pub oauth2_public_key: Option<String>,
    pub scope: Option<String>,
    pub resource_provider_type: ResourceProviderType,
    pub created: PrimitiveDateTime,
    pub updated: PrimitiveDateTime,
}

impl TryFrom<&PgRow> for ResourceProvider {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("provider_type");

        Ok(ResourceProvider {
            id: row.get("id"),
            name: row.get("name"),
            client_id: row.get("client_id"),
            client_secret: row.get("client_secret"),
            identity_redirect_url: row.get("identity_redirect_url"),
            oauth2_token_url: row.get("oauth2_token_url"),
            oauth2_jwks_url: row.get("oauth2_jwks_url"),
            oidc_user_info_url: row.get("oidc_user_info_url"),
            oauth2_public_key: row.get("oauth2_public_key"),
            scope: row.get("scope"),
            resource_provider_type: ResourceProviderType::from_str(provider)?,
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum ResourceProviderType {
    TaccTapis,
}

impl FromStr for ResourceProviderType {
    type Err = ServiceError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "tacc_tapis" => Ok(ResourceProviderType::TaccTapis),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", s))),
        }
    }
}

pub async fn db_get_resource_providers<'a>(
    tx: &mut PgTransaction<'a>,
) -> Result<HashSet<ResourceProvider>> {
    let rp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type, created, updated from resource_providers",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut rps: Vec<ResourceProvider> = vec![];
    for row in &rp_query_result {
        rps.push(ResourceProvider::try_from(row)?);
    }
    Ok(HashSet::from_iter(rps))
}

pub async fn db_get_resource_provider_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    provider_id: &String,
) -> Result<ResourceProvider> {
    let rp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type, created, updated from resource_providers 
                     where id = $1",
    )
        .bind(provider_id)
        .fetch_one(&mut **tx)
        .await?;
    Ok(ResourceProvider::try_from(&rp_query_result)?)
}
