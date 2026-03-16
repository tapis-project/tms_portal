use std::collections::HashMap;
use axum::Json;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use reqwest::{StatusCode};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct TmsApiErrorResult {
    pub message: String,
}
#[derive(Debug, Clone)]
pub enum Entity<T> where T: Serialize {
    Error(Json<TmsApiErrorResult>),
    Success(Json<T>),
}
pub struct HttpResponse<T> where T: Serialize{
    pub status: StatusCode,
    pub headers: Option<HashMap<String, String>>,
    pub entity: Option<Entity<T>>,
}
impl <T> From<StatusCode> for HttpResponse<T> where T: Serialize {
    fn from(status: StatusCode) -> Self {
        HttpResponse { status, headers: None, entity: None }
    }
}
impl <T> From<(StatusCode, Entity<T>)> for HttpResponse<T> where T: Serialize {
    fn from(value: (StatusCode, Entity<T>)) -> Self {
        HttpResponse {
            status: value.0,
            headers: None,
            entity: Some(value.1)
        }
    }
}
impl <T> From<(StatusCode, HashMap<String, String>, Entity<T>)> for HttpResponse<T> where T: Serialize {
    fn from(value: (StatusCode, HashMap<String, String>, Entity<T>)) -> Self {
        HttpResponse {
            status: value.0,
            headers: Some(value.1),
            entity: Some(value.2)
        }
    }
}
impl <T> From<(StatusCode, HashMap<String, String>)> for HttpResponse<T> where T: Serialize {
    fn from(value: (StatusCode, HashMap<String, String>)) -> Self {
        HttpResponse {
            status: value.0,
            headers: Some(value.1),
            entity: None
        }
    }
}
impl <T> From<TmsApiErrorResult> for Entity<T> where T:Serialize {
    fn from(value: TmsApiErrorResult) -> Self {
        Entity::Error(Json(value))
    }
}
impl <T> IntoResponse for HttpResponse<T> where T: Serialize {
    fn into_response(self) -> Response {
        let body = if self.entity.is_some() {
            match self.entity.unwrap() {
                Entity::Success(entity) => {
                    Body::from(serde_json::ser::to_string(&entity.0).unwrap())
                }
                Entity::Error(Json(error)) => {
                    Body::from(serde_json::to_string(&error).unwrap())
                }

            }
        } else {
            Body::empty()
        };

        let mut builder = Response::builder()
            .status(self.status);

        if self.headers.is_some() {
            for header in self.headers.unwrap().iter() {
                builder = builder.header(header.0.as_str(), header.1.as_str());
            }
        }

        builder.body(body).unwrap()
    }
}