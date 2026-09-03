use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::delegation::Delegation;

impl From<&PgRow> for Delegation {
    fn from(row: &PgRow) -> Self {
        Delegation {
            id: row.get("id"),
            client_id: row.get("client_id"),
            rp_account: row.get("rp_account"),
            expires_at: row.get("expires_at"),
            created: row.get("created"),
            updated: row.get("updated"),
            tms_identity: row.get("tms_identity"),
            rp_id: row.get("rp_id"),
        }
    }
}
pub async fn db_insert_delegation<'a>(
    tx: &mut PgTransaction<'a>,
    client_id: & String, rp_account: & String, expires_at: &DateTime<Utc>,
    tms_identity: &String, rp_id: &String) -> anyhow::Result<Delegation> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let row = query("insert into delegations (client_id, rp_account, expires_at,
                         tms_identity, rp_id) values ($1, $2, $3, $4, $5) returning *")
        .bind(client_id)
        .bind(rp_account)
        .bind(expires_at)
        .bind(tms_identity)
        .bind(rp_id)
        .fetch_one(&mut **tx)
        .await?;
    Ok(Delegation::from(&row))
}
