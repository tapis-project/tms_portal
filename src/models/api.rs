use axum::body::Body;
use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone)]
pub struct TmsApiErrorResult {
    pub message: String,
}
#[derive(Debug, Clone)]
pub enum Entity<T>
where
    T: Serialize,
{
    Error(Json<TmsApiErrorResult>),
    Success(Json<T>),
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
    pub(crate) status_code: StatusCode,
    pub(crate) entity: Option<Entity<T>>,
    pub(crate) headers: Option<HashMap<String, String>>,
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
        let body = if self.entity.is_some() {
            match self.entity.unwrap() {
                Entity::Success(entity) => {
                    Body::from(serde_json::ser::to_string(&entity.0).unwrap())
                }
                Entity::Error(Json(error)) => Body::from(serde_json::to_string(&error).unwrap()),
            }
        } else {
            Body::empty()
        };

        let mut builder = Response::builder().status(self.status_code);

        if self.headers.is_some() {
            for header in self.headers.unwrap().iter() {
                builder = builder.header(header.0.as_str(), header.1.as_str());
            }
        }

        builder.body(body).unwrap()
    }
}

impl<T> From<TmsApiErrorResult> for Entity<T>
where
    T: Serialize,
{
    fn from(value: TmsApiErrorResult) -> Self {
        Entity::Error(Json(value))
    }
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}
