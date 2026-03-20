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
    pub oauth2_jwks_url: String,
    pub user_info_url: String,
    pub scope: String,
    pub created: PrimitiveDateTime,
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
            user_info_url: row.get("user_info_url"),
            scope: row.get("scope"),
            created: row.get("created"),
            updated: row.get("updated"),
        }
    }
}

pub async fn get_idps<'a>(tx: &mut PgTransaction<'a>) -> Result<HashSet<Idp>> {
    let idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, user_info_url, scope,
                     created, updated from idps",
    )
    .fetch_all(&mut **tx)
    .await?;

    let idp_collection: Vec<Idp> = idp_query_result
        .iter()
        .map(|row| Idp::from(row.clone()))
        .collect();
    dbg!(&idp_collection);
    Ok(HashSet::from_iter(idp_collection.into_iter()))
}

pub async fn get_idp_by_id<'a>(tx: &mut PgTransaction<'a>, id: &String) -> Result<Idp> {
    let row = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, user_info_url, scope,
                     created, updated from idps",
    )
    .fetch_one(&mut **tx)
    .await?;

    dbg!(&row);
    Ok(Idp::from(&row))
}
