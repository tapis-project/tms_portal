use tms_lib::utils::service_error::ServiceError::NotFound;
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::tms_identity::TmsIdentity;

impl From<&PgRow> for TmsIdentity {
    fn from(row: &PgRow) -> Self {
        TmsIdentity {
            seq_id: row.get("seq_id"),
            tms_identity: row.get("tms_identity"),
            created: row.get("created"),
        }
    }
}
pub async fn db_get_tms_identity<'a>(
    tx: &mut PgTransaction<'a>,
    tms_identity: &String,
) -> anyhow::Result<TmsIdentity> {
    let row = query("select * from tms_identity where tms_identity = $1")
        .bind(tms_identity)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => NotFound(format!("Tms Identity {} not found", tms_identity)).into(),
            _ => anyhow::anyhow!(error),
        })?;
    Ok(TmsIdentity::from(&row))
}

pub async fn db_insert_tms_identity_if_absent<'a>(
    tx: &mut PgTransaction<'a>,
    tms_identity: &String) -> anyhow::Result<TmsIdentity> {
    // on conflict do update ... really does nothing, but it ensures that a record is returned
    let row = query("insert into tms_identities (tms_identity) values ($1) on conflict(tms_identity) do update set tms_identity = EXCLUDED.tms_identity returning *")
        .bind(tms_identity)
        .fetch_one(&mut **tx)
        .await?;
    Ok(TmsIdentity::from(&row))
}
