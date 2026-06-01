use crate::models::login_api::IdpProvider;
use crate::services::service_error::ServiceError::NotFound;
use anyhow::Result;
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Idp {
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
    pub provider: IdpProvider,
    pub created: PrimitiveDateTime,
    pub updated: PrimitiveDateTime,
}

impl TryFrom<&PgRow> for Idp {
    type Error = anyhow::Error;
    fn try_from(row: &PgRow) -> anyhow::Result<Self, Self::Error> {
        let provider: &str = row.get("provider");

        Ok(Idp {
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
            provider: IdpProvider::from_str(provider)?,
            created: row.get("created"),
            updated: row.get("updated"),
        })
    }
}

pub async fn db_get_idps<'a>(tx: &mut PgTransaction<'a>) -> Result<HashSet<Idp>> {
    let idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider, created, updated from idps",
    )
    .fetch_all(&mut **tx)
    .await?;

    let mut idps: Vec<Idp> = vec![];
    for row in &idp_query_result {
        idps.push(Idp::try_from(row)?);
    }
    Ok(HashSet::from_iter(idps))
}

pub async fn db_get_idp_by_id<'a>(tx: &mut PgTransaction<'a>, id: &String) -> Result<Idp> {
    let row = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                     oauth2_public_key, scope, provider, created, updated from idps where id = $1",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Idp id {} not found", id)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    dbg!(&row);
    Ok(Idp::try_from(&row)?)
}
