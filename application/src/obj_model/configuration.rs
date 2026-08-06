use std::borrow::Cow;
use log::trace;
use serde::{Deserialize};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct StateKey {
    pub kid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub default_expiration_minutes: String,
    pub signing_key_kid: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    pub login_oauth_provider: String,
}
#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeConfig {
    pub config_directory: String,
    pub logging_config_file_name: String,
    // TODO:  put db host and db port in here?
}
#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    pub base_url: String,
    pub identity_provider_callback_endpoint: String,
    pub resource_provider_callback_endpoint: String,
    pub oauth_provider_callback_endpoint: String,
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
    pub fn get_oauth_provider_callback_url(&self) -> String {
        self.make_tms_url(&self.base_url, &self.oauth_provider_callback_endpoint)
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

        trace!("Base Url String: {0}", base_url_string);
        trace!("Relative Path String: {0}", relative_path_string);
        if let Ok(base_url) = Url::parse(base_url_string) {
            trace!("Base Url: {0}", base_url);
            if let Ok(full_url) = base_url.join(relative_path_string.as_ref()) {
                trace!("Full Url: {0}", full_url);
                return full_url.to_string();
            };
        };
        String::default()
    }
}

