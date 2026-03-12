use axum::Json;
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum TmsApiError {
    NotFound
}
#[derive(Debug, Serialize)]
pub struct TmsApiErrorResult {
    pub message: String,
}
#[derive(Debug, Serialize)]
pub struct TmsApiResponse<T: Serialize> {
    // more fields will probably go here
    pub result: Result<T, TmsApiErrorResult>
}

impl <T:Serialize> TmsApiResponse<T> {
    pub fn new(result: Result<T, TmsApiErrorResult>) -> Self {
        Self { result }
    }
}
impl Into<StatusCode> for TmsApiError {
    fn into(self) -> StatusCode {
        match self {
            TmsApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

pub enum Entity<T> where T: Serialize {
    Error(Json<TmsApiErrorResult>),
    Success(Json<T>),
}

pub struct HttpResponse<T> where T: Serialize{
    pub status: StatusCode,
    pub entity: Entity<T>,
}

impl <T> IntoResponse for HttpResponse<T> where T: Serialize {
    fn into_response(self) -> Response {
        match self.entity {
            Entity::Error(err) => (self.status, err).into_response(),
            Entity::Success(success) => (self.status, success).into_response(),
        }
    }
}