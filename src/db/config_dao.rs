use crate::services::service_error::ServiceError::Internal;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use std::collections::HashMap;

const CONFIG_NAME_STATE_KEY: &str = "state_key";
const CONFIG_NAME_HTTP_CONFIG: &str = "http_config";
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateKey {
    pub public_key: String,
    pub private_key: String,
}
impl TryFrom<&PgRow> for StateKey {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        let result: HashMap<String, StateKey> = serde_json::from_value(row.get("config_value"))?;
        if let Some(key) = result.get("current") {
            return Ok(key.clone());
        }
        Err(Internal("Unable to find key for internal state".to_string()).into())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpConfig {
    pub callback_url: String,
}
impl TryFrom<&PgRow> for HttpConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}

pub async fn db_get_state_key<'a>(tx: &mut PgTransaction<'a>) -> anyhow::Result<StateKey> {
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
