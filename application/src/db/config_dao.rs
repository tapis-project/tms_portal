use anyhow::Error;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::{query, PgTransaction, Row};
use std::borrow::Cow;
use tracing::debug;
use url::Url;

const CONFIG_NAME_STATE_KEY: &str = "state_key";
const CONFIG_NAME_HTTP_CONFIG: &str = "http_config";
const CONFIG_NAME_OAUTH_CONFIG: &str = "oauth_config";
const CONFIG_NAME_JWT_CONFIG: &str = "jwt_config";
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateKey {
    pub kid: String,
}
impl TryFrom<&PgRow> for StateKey {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        let result: StateKey = serde_json::from_value(row.get("config_value"))?;
        Ok(result)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtConfig {
    pub default_expiration_minutes: String,
    pub signing_key_kid: String,
}
impl TryFrom<&PgRow> for JwtConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OAuthConfig {
    pub login_oauth_provider: String,
}
impl TryFrom<&PgRow> for OAuthConfig {
    type Error = Error;

    fn try_from(row: &PgRow) -> Result<Self, Self::Error> {
        Ok(serde_json::from_value(row.get("config_value"))?)
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpConfig {
    pub base_url: String,
    pub identity_provider_callback_endpoint: String,
    pub resource_provider_callback_endpoint: String,
    pub token_endpoint: String,
    pub authorization_endpoint: String,
    pub revocation_endpoint: String,
    pub jwks_endpoint: String,
}

impl HttpConfig {
    pub fn get_identity_provider_callback_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.identity_provider_callback_endpoint)
    }
    pub fn get_resource_provider_callback_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.resource_provider_callback_endpoint)
    }
    pub fn get_token_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.token_endpoint)
    }
    pub fn get_authorization_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.authorization_endpoint)
    }
    pub fn get_revocation_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.revocation_endpoint)
    }
    pub fn get_jwks_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.jwks_endpoint)
    }
    pub fn make_tms_url(&self, base_url: &str, relative_path: &str) -> String {
        // trim and ensure string ends with a "/"
        let base_url_string = base_url.trim_end_matches("/");

        // trim and ensure string doesnt start with a "/"
        let mut relative_path_string = Cow::from(relative_path.trim());
        if !relative_path_string.starts_with("/") {
            relative_path_string = Cow::from(format!("/{}", relative_path_string))
        };

        debug!("Base Url String: {0}", base_url_string);
        debug!("Relative Path String: {0}", relative_path_string);
        if let Ok(base_url) = Url::parse(base_url_string) {
            debug!("Base Url: {0}", base_url);
            if let Ok(full_url) = base_url.join(relative_path_string.as_ref()) {
                debug!("Full Url: {0}", full_url);
                return full_url.to_string();
            };
        };
        String::default()
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
