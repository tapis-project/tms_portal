use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use std::collections::HashMap;

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


