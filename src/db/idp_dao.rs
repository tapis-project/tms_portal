use crate::db::config_dao::{get_client_id, get_client_secret};
use crate::models::tms_internal::TmsResult;
use sqlx::types::time::PrimitiveDateTime;
use sqlx::{query, PgPool, Row};
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
pub async fn get_idps<'a>(pool: &PgPool) -> TmsResult<HashSet<Idp>> {
    let mut tx = pool.begin().await.unwrap();
    let mut idp_query_result = query(
        "select id, name, client_id, client_secret, identity_redirect_url,
             oauth2_token_url, oauth2_jwks_url, user_info_url, scope,
             created, updated from idps",
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap();

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
    // let idp_query_result: Vec<Idp> = query("select * from idps")
    //     .fetch_all(transaction)
    //     .await
    //     .unwrap();

    dbg!(idp_query_result);

    let cilogon_idp = Idp {
        id: "cilogon_idp".to_string(),
        name: "CILogon IDP".to_string(),
        client_id: get_client_id(),
        client_secret: get_client_secret(),
        identity_redirect_url: "https://cilogon.org/authorize".to_string(),
        oauth2_token_url: "https://cilogon.org/oauth2/token".to_string(),
        oauth2_jwks_url: "https://cilogon.org/oauth2/certs".to_string(),
        user_info_url: "https://cilogon.org/oauth2/userinfo".to_string(),
        scope: "openid profile email org.cilogon.userinfo".to_string(),
        created: PrimitiveDateTime::MIN,
        updated: PrimitiveDateTime::MIN,
    };

    let mut idps = HashSet::new();
    idps.insert(cilogon_idp);
    Ok(idps)
}

pub async fn get_idp_by_id(pool: &PgPool, id: &String) -> TmsResult<Idp> {
    let idps = get_idps(pool).await?;
    let idp = idps
        .iter()
        .find(|idp| -> bool { idp.id.eq(id) })
        .ok_or_else(|| format!("Could not find id: {}", id))?;
    Ok(idp.to_owned())
}
