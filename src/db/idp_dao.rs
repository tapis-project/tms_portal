use crate::services::service_error::ServiceError::NotFound;
use anyhow::Result;
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgTransaction, Row};
use std::collections::HashSet;

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
    pub created: PrimitiveDateTime,
    pub oauth2_public_key: Option<String>,
    pub scope: Option<String>,
    pub updated: PrimitiveDateTime,
}

impl From<&PgRow> for Idp {
    fn from(row: &PgRow) -> Self {
        Idp {
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
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn db_get_idps<'a>(tx: &mut PgTransaction<'a>) -> Result<HashSet<Idp>> {
    let idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, 
                     oauth2_public_key, scope, created, updated from idps",
    )
    .fetch_all(&mut **tx)
    .await?;

    let idp_collection: Vec<Idp> = idp_query_result.iter().map(|row| Idp::from(row)).collect();
    //    dbg!(&idp_collection);
    Ok(HashSet::from_iter(idp_collection.into_iter()))
}

pub async fn db_get_idp_by_id<'a>(tx: &mut PgTransaction<'a>, id: &String) -> Result<Idp> {
    let row = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, oidc_user_info_url, 
                     oauth2_public_key, scope, created, updated from idps where id = $1",
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|error| match error {
        sqlx::Error::RowNotFound => NotFound(format!("Idp id {} not found", id)).into(),
        _ => anyhow::anyhow!(error),
    })?;
    dbg!(&row);
    Ok(Idp::from(&row))
}
