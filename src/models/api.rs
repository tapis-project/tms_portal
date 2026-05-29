use crate::services::service_error::ServiceError;
use axum::body::Body;
use axum::http;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Entity<T>
where
    T: Serialize,
{
    Success(T),
}

pub struct TmsResponseBuilder<T>
where
    T: Serialize,
{
    status_code: StatusCode,
    entity: Option<T>,
    headers: Option<HashMap<String, String>>,
}

impl<T> TmsResponseBuilder<T>
where
    T: Serialize,
{
    pub fn entity(mut self, entity: T) -> TmsResponseBuilder<T> {
        self.entity = Some(entity);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> TmsResponseBuilder<T> {
        self.headers = Some(headers);
        self
    }

    pub fn build(self) -> TmsResponse<T> {
        TmsResponse {
            status_code: self.status_code,
            headers: self.headers,
            entity: self.entity,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponseBody<T>
where
    T: Serialize,
{
    status: String,
    result: Option<T>,
}

pub struct TmsResponse<T>
where
    T: Serialize,
{
    pub status_code: StatusCode,
    pub entity: Option<T>,
    pub headers: Option<HashMap<String, String>>,
}

impl<T> TmsResponse<T>
where
    T: Serialize,
{
    pub fn builder(status_code: StatusCode) -> TmsResponseBuilder<T> {
        TmsResponseBuilder {
            status_code,
            entity: None,
            headers: None,
        }
    }
}

impl<T> IntoResponse for TmsResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let result = match self.entity {
            Some(entity) => ApiResponseBody {
                status: self.status_code.to_string(),
                result: Some(entity),
            },

            None => ApiResponseBody {
                status: self.status_code.to_string(),
                result: None,
            },
        };

        let mut builder = Response::builder().status(self.status_code);

        if let Some(headers) = self.headers {
            for (key, value) in headers.iter() {
                builder = builder.header(key, value);
            }
        } else {
            builder = builder.header(http::header::CONTENT_TYPE, "application/json");
        };

        if let Ok(json_string) = serde_json::ser::to_string(&result) {
            if let Ok(response) = builder.body(Body::from(json_string)) {
                return response;
            };
        };

        internal_error_response("Unable to build response")
    }
}
pub fn internal_error_response(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhoAmIResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Value>,
    pub username: Value,
    #[serde(skip_serializing_if = "Option::is_none", rename = "idpDisplayName")]
    pub idp_display_name: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub enum IdpProvider {
    Globus,
}

impl FromStr for IdpProvider {
    type Err = ServiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "globus" => Ok(IdpProvider::Globus),
            _ => Err(ServiceError::Internal(format!("Unknown provider {0}", s))),
        }
    }
}
