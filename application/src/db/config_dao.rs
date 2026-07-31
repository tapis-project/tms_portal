use anyhow::Error;
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use crate::obj_model::configuration::{HttpConfig, JwtConfig, OAuthConfig, RuntimeConfig, StateKey};

const CONFIG_NAME_STATE_KEY: &str = "state_key";
const CONFIG_NAME_HTTP_CONFIG: &str = "http_config";
const CONFIG_NAME_OAUTH_CONFIG: &str = "oauth_config";
const CONFIG_NAME_JWT_CONFIG: &str = "jwt_config";
const CONFIG_NAME_RUNTIME_CONFIG: &str = "runtime_config";
impl TryFrom<&PgRow> for StateKey {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        let result: StateKey = serde_json::from_value(row.get("config_value"))?;
        Ok(result)
    }
}
impl TryFrom<&PgRow> for JwtConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}
impl TryFrom<&PgRow> for OAuthConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}


impl TryFrom<&PgRow> for RuntimeConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}

impl TryFrom<&PgRow> for HttpConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}

pub async fn db_get_state_key_id<'a>(tx: &mut PgTransaction<'a>) -> anyhow::Result<StateKey> {
    let row = query("select config_value from configuration where config_name = $1")
        .bind(CONFIG_NAME_STATE_KEY)
        .fetch_one(&mut **tx)
        .await?;
    dbg!(&row);
    StateKey::try_from(&row)
}

pub async fn db_get_http_config(tx: &mut PgTransaction<'_>) -> anyhow::Result<HttpConfig> {
    let row = query("select config_value from configuration where config_name = $1")
        .bind(CONFIG_NAME_HTTP_CONFIG)
        .fetch_one(&mut **tx)
        .await?;
    dbg!(&row);
    HttpConfig::try_from(&row)
}

pub async fn db_get_oauth_config(tx: &mut PgTransaction<'_>) -> anyhow::Result<OAuthConfig> {
    let row = query("select config_value from configuration where config_name = $1")
        .bind(CONFIG_NAME_OAUTH_CONFIG)
        .fetch_one(&mut **tx)
        .await?;
    dbg!(&row);
    OAuthConfig::try_from(&row)
}

pub async fn db_get_jwt_config(tx: &mut PgTransaction<'_>) -> anyhow::Result<JwtConfig> {
    let row = query("select config_value from configuration where config_name = $1")
        .bind(CONFIG_NAME_JWT_CONFIG)
        .fetch_one(&mut **tx)
        .await?;
    dbg!(&row);
    JwtConfig::try_from(&row)
}
pub async fn db_get_runtime_config(tx: &mut PgTransaction<'_>) -> anyhow::Result<RuntimeConfig> {
    let row = query("select config_value from configuration where config_name = $1")
        .bind(CONFIG_NAME_RUNTIME_CONFIG)
        .fetch_one(&mut **tx)
        .await?;
    dbg!(&row);
    RuntimeConfig::try_from(&row)
}
