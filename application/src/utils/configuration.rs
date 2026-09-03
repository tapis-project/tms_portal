use sqlx::PgPool;
use crate::db::config_dao::{db_get_delegation_policy_config, db_get_http_config, db_get_jwt_config, db_get_oauth_config, db_get_runtime_config};
use crate::obj_model::configuration::{DelegationPolicyConfig, HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig};

pub struct Configuration {
    pub http_config: HttpConfig,
    pub oauth_config: OAuthConfig,
    pub jwt_config: JwtConfig,
    pub runtime_config: RuntimeConfig,
    pub delegation_policy_config: DelegationPolicyConfig,
}

impl Configuration {
    pub async fn get(db_pool:&PgPool) -> anyhow::Result<Configuration> {
        let mut tx = db_pool.begin().await?;
        let http_config = db_get_http_config(&mut tx).await?;
        let oauth_config = db_get_oauth_config(&mut tx).await?;
        let jwt_config = db_get_jwt_config(&mut tx).await?;
        let runtime_config = db_get_runtime_config(&mut tx).await?;
        let delegation_policy_config = db_get_delegation_policy_config(&mut tx).await?;
        tx.commit().await?;
        Ok(Configuration {
            http_config,
            oauth_config,
            jwt_config,
            runtime_config,
            delegation_policy_config,
        })
    }
}