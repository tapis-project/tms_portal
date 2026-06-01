use crate::services::service_error::ServiceError;
use crate::services::service_error::ServiceError::NotFound;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct IdentityProvider {
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
    pub identity_provider_type: IdentityProviderType,
    pub created: PrimitiveDateTime,
    pub updated: PrimitiveDateTime,
}

impl TryFrom<&PgRow> for IdentityProvider {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("provider_type");

        Ok(IdentityProvider {
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
            identity_provider_type: IdentityProviderType::from_str(provider)?,
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum IdentityProviderType {
    Globus,
}

impl FromStr for IdentityProviderType {
    type Err = ServiceError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "globus" => Ok(IdentityProviderType::Globus),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", s))),
        }
    }
}

pub async fn db_get_idps<'a>(tx: &mut PgTransaction<'a>) -> Result<HashSet<IdentityProvider>> {
    let idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type, created, updated from identity_providers",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut idps: Vec<IdentityProvider> = vec![];
    for row in &idp_query_result {
        idps.push(IdentityProvider::try_from(row)?);
    }
    Ok(HashSet::from_iter(idps))
}

pub async fn db_get_idp_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
) -> Result<IdentityProvider> {
    let row = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type, created, updated from identity_providers where id = $1",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Idp id {} not found", id)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    dbg!(&row);
    Ok(IdentityProvider::try_from(&row)?)
}
