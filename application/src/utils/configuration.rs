use sqlx::PgPool;
use crate::db::config_dao::{db_get_http_config, db_get_jwt_config, db_get_oauth_config, HttpConfig, JwtConfig, OAuthConfig};

pub struct Configuration {
    pub http_config: HttpConfig,
    pub oauth_config: OAuthConfig,
    pub jwt_config: JwtConfig ,
}

impl Configuration {
    pub async fn get(db_pool:&PgPool) -> anyhow::Result<Configuration> {
        let mut tx = db_pool.begin().await?;
        let http_config = db_get_http_config(&mut tx).await?;
        let oauth_config = db_get_oauth_config(&mut tx).await?;
        let jwt_config = db_get_jwt_config(&mut tx).await?;
        tx.commit().await?;
        Ok(Configuration {
            http_config,
            oauth_config,
            jwt_config,
        })
    }
}