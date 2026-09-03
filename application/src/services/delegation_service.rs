use chrono::Utc;
use sqlx::{PgPool, PgTransaction};
use chrono::Duration;
use tms_lib::utils::service_error::ServiceError::NotFound;
use crate::db::delegations_dao::db_insert_delegation;
use crate::db::resource_provider_logins_dao::{db_get_resource_provider_login};
use crate::obj_model::delegation::Delegation;
use crate::utils::configuration::Configuration;

pub async fn add_delegation(
    db_pool: &PgPool, tms_identity:&String, client_id:&String, rp_id:&String, rp_account:&String
) -> anyhow::Result<Delegation> {
    let configuration = Configuration::get(&db_pool).await?;
    let expiration = &configuration.delegation_policy_config.get_delegation_expiration()?;
    let mut tx = db_pool.begin().await?;

    // First check that the rp_id/rp_account are linked, enabled, and not expired
    let _ = match ensure_recent_rp_login(&mut tx, tms_identity, rp_account, rp_id, configuration.delegation_policy_config.delegation_max_mins_since_login).await {
        Ok(recent_login) => Some(recent_login),
        Err(e) => return Err(e.into()),
    };

    let delegation = db_insert_delegation(&mut tx, client_id, rp_account,
                                          &expiration,
                                          tms_identity, rp_id).await?;
    tx.commit().await?;
    Ok(delegation)
}

async fn ensure_recent_rp_login<'a>(tx: &mut PgTransaction<'a>, tms_identity:&String,
                                    rp_account:&String, rp_id:&String, max_mins_since_login:i64
) -> anyhow::Result<()> {
    let last_allowed_login = Utc::now() - Duration::minutes(max_mins_since_login);
    if let Some(_) = db_get_resource_provider_login(tx, tms_identity,
                                                         rp_account, rp_id, last_allowed_login).await? {
        return Ok(());
    } else {
        return Err(NotFound("User must authenticate to delegate".to_string()).into());
    }
}