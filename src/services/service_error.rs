use crate::models::general_api::TmsResponse;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

pub struct AppError(anyhow::Error);
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Method Not allowed: {0}")]
    MethodNotAllowed(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let error_tuple = if let Some(error) = self.0.downcast_ref::<ServiceError>() {
            match error {
                ServiceError::Internal(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", error))
                }
                ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, format!("{:#}", error)),
                ServiceError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, format!("{:#}", error)),
                ServiceError::BadRequest(_) => (StatusCode::BAD_REQUEST, format!("{:#}", error)),
                ServiceError::MethodNotAllowed(_) => {
                    (StatusCode::METHOD_NOT_ALLOWED, format!("{:#}", error))
                }
            }
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("(Generic): {:#}", self.0),
            )
        };

        // build a TmsResponse object, and convert that into a Response
        TmsResponse::builder(error_tuple.0)
            .entity(error_tuple.1)
            .build()
            .into_response()
    }
}
