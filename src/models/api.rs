use crate::models::api::Entity::ServiceError;
use crate::models::tms_internal::TmsServiceError;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

impl Display for TmsServiceError {
    // TODO: this should not be needed at some point soon.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "An error has occurred!")
    }
}

#[derive(Debug, Clone)]
pub enum Entity<T>
where
    T: Serialize,
{
    ServiceError(TmsServiceError),
    Success(T),
}
impl<T> From<TmsServiceError> for Entity<T>
where
    T: Serialize,
{
    fn from(value: TmsServiceError) -> Self {
        ServiceError(value)
    }
}

pub struct TmsResponseBuilder<T>
where
    T: Serialize,
{
    status_code: StatusCode,
    entity: Option<Entity<T>>,
    headers: Option<HashMap<String, String>>,
}

impl<T> TmsResponseBuilder<T>
where
    T: Serialize,
{
    pub fn entity(mut self, entity: Entity<T>) -> TmsResponseBuilder<T> {
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
pub struct TmsResponse<T>
where
    T: Serialize,
{
    pub status_code: StatusCode,
    pub entity: Option<Entity<T>>,
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
        let body = match self.entity {
            Some(Entity::Success(entity)) => {
                if let Ok(json_string) = serde_json::ser::to_string(&entity) {
                    Body::from(json_string)
                } else {
                    return internal_error_response("Unable to serialize entity");
                }
            }
            Some(Entity::ServiceError(error)) => {
                if let Ok(json_string) = serde_json::ser::to_string(&error) {
                    Body::from(json_string)
                } else {
                    return internal_error_response("Unable to serialize entity");
                }
            }
            None => Body::default(),
        };

        let mut builder = Response::builder().status(self.status_code);

        if let Some(headers) = self.headers {
            for (key, value) in headers.iter() {
                builder = builder.header(key, value);
            }
        }

        if let Ok(response) = builder.body(body) {
            response
        } else {
            internal_error_response("Unable to build response")
        }
    }
}
pub fn internal_error_response(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}
