use tms_lib::utils::service_error::{ ServiceError, ServiceError::NotFound };
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{query, Error, PgTransaction, Row};
use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use tms_lib::utils::service_error::ServiceError::BadRequest;
/*
Identity providers can be for either resources or for logins.  There's a boolean for
the support of each - supports_login, supports_resources.  I guess in retrospect it should
have been named login_allowed and resources_allowed because it's not about support, but
rather if we allow it.
 */

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct IdentityProvider {
    pub uuid: Option<Uuid>,
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
    pub supports_login: bool,
    pub supports_resources: bool,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl TryFrom<&PgRow> for IdentityProvider {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("provider_type");
        Ok(IdentityProvider {
            uuid: Some(row.get("uuid")),
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
            supports_login: row.get("supports_login"),
            supports_resources: row.get("supports_resources"),
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}
#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum IdentityProviderType {
    Globus,
    TaccTapis,
}

impl FromStr for IdentityProviderType {
    type Err = ServiceError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        /*
                match value {
            v if v.eq_ignore_ascii_case("authorization_code") => Ok(GrantType::AuthorizationCode),
            v if v.eq_ignore_ascii_case("refresh_token") => Ok(GrantType::RefreshToken),
            _ => Err(BadRequest(format!("Unknown GrantType {}", value))),
        }

         */
        match value {
            idp_type if idp_type.eq_ignore_ascii_case("globus") => Ok(IdentityProviderType::Globus),
            idp_type if idp_type.eq_ignore_ascii_case("tacc_tapis") => Ok(IdentityProviderType::TaccTapis),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", value))),
        }
    }
}

impl Display for IdentityProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityProviderType::Globus => write!(f, "globus"),
            IdentityProviderType::TaccTapis => write!(f, "tacc_tapis"),
        }
    }
}

/*
Returns the list of identity providers that support login
 */
pub async fn db_get_login_providers<'a>(
    tx: &mut PgTransaction<'a>,
) -> Result<HashSet<IdentityProvider>> {
    let idp_query_result = query(
        "select uuid, id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type,
                     supports_login, supports_resources, created, updated
                     from identity_providers where supports_login = true",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut idps: Vec<IdentityProvider> = vec![];
    for row in &idp_query_result {
        idps.push(IdentityProvider::try_from(row)?);
    }
    Ok(HashSet::from_iter(idps))
}

/*
Returns an identity provider by id if it supports login
 */
pub async fn db_get_login_provider_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    id: &String,
) -> Result<IdentityProvider> {
    let row = query(
        "select uuid, id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider_type,
                     supports_login, supports_resources, created, updated
                     from identity_providers where id = $1 and supports_login = true",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Idp id {} not found", id)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    dbg!(&row);
    IdentityProvider::try_from(&row)
}
/*
Returns the list of identity providers that support resources
 */
pub async fn db_get_resource_providers<'a>(
    tx: &mut PgTransaction<'a>, tms_identity: &String, linked_only: &bool
) -> Result<HashSet<IdentityProvider>> {
    let rp_query_result = match linked_only {
        true => {
            query(
                "select * from identity_providers as ip
                      INNER JOIN resource_provider_account_logins
                      AS rpal ON ip.uuid = rpal.resource_provider_uuid where supports_resources = true
                      and tms_identity = $1",
            ).bind(tms_identity)
                .fetch_all(&mut **tx)
                .await?
        }
        false => {
            query(
                "select * from identity_providers
                     where supports_resources = true",
            )
                .fetch_all(&mut **tx)
                .await?
        }
    };

    let mut rps: Vec<IdentityProvider> = vec![];
    for row in &rp_query_result {
        rps.push(IdentityProvider::try_from(row)?);
    }
    Ok(HashSet::from_iter(rps))
}
/*
Returns an identity provider by id if it supports resources
 */
pub async fn db_get_resource_provider_by_id<'a>(
    tx: &mut PgTransaction<'a>,
    provider_id: &String,
) -> Result<IdentityProvider> {
    match query(
        "select * from identity_providers
                     where id = $1 and supports_resources = true",
    )
    .bind(provider_id)
    .fetch_one(&mut **tx)
    .await {
        Ok(row) => Ok(IdentityProvider::try_from(&row)?),
        Err(Error::RowNotFound) => Err(BadRequest("Resource provider not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}
pub async fn db_get_resource_provider_by_uuid<'a>(
    tx: &mut PgTransaction<'a>,
    provider_uuid: &Uuid,
) -> Result<IdentityProvider> {
    match query(
        "select * from identity_providers
                     where uuid = $1 and supports_resources = true",
    )
        .bind(provider_uuid)
        .fetch_one(&mut **tx)
        .await {
        Ok(row) => Ok(IdentityProvider::try_from(&row)?),
        Err(Error::RowNotFound) => Err(BadRequest("Resource provider not found".to_string()).into()),
        Err(error) => Err(anyhow!(error)),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::db::identity_provider_dao::IdentityProviderType;

    #[test]
    fn test_idp_type_from_str() {
        assert!(IdentityProviderType::from_str("test").is_err());
        assert_eq!(IdentityProviderType::from_str("Globus").unwrap(), IdentityProviderType::Globus);
        assert_eq!(IdentityProviderType::from_str("tacc_tapis").unwrap(), IdentityProviderType::TaccTapis);
    }
}