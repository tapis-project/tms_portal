use crate::models::tms_internal::TmsResult;
use crate::models::tms_internal::TmsServiceError::{DatabaseError, InternalError};
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgPool, PgTransaction, Row};
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

pub async fn get_idps<'a>(tx: &mut PgTransaction<'a>) -> TmsResult<HashSet<Idp>> {
    let idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
                     oauth2_token_url, oauth2_jwks_url, user_info_url, scope,
                     created, updated from idps",
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| DatabaseError(e.to_string()))?;

    let idp_collection: Vec<Idp> = idp_query_result
        .iter()
        .map(|row| Idp {
            id: row.get(0),
            name: row.get(1),
            client_id: row.get(2),
            client_secret: row.get(3),
            identity_redirect_url: row.get(4),
            oauth2_token_url: row.get(5),
            oauth2_jwks_url: row.get(6),
            user_info_url: row.get(7),
            scope: row.get(8),
            created: row.get(9),
            updated: row.get(10),
        })
        .collect();
    dbg!(&idp_collection);
    Ok(HashSet::from_iter(idp_collection.into_iter()))
}

pub async fn get_idp_by_id(pool: &PgPool, id: &String) -> TmsResult<Idp> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| DatabaseError(e.to_string()))?;
    let idps = get_idps(&mut tx).await?;
    let idp = idps
        .iter()
        .find(|idp| -> bool { idp.id.eq(id) })
        .ok_or_else(|| InternalError(format!("Could not find id: {}", id)))?;
    tx.commit()
        .await
        .map_err(|e| DatabaseError(e.to_string()))?;
    Ok(idp.to_owned())
}
