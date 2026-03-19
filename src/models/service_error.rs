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
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        if let Some(error) = self.0.downcast_ref::<ServiceError>() {
            match error {
                ServiceError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Formated Interal: {:#}", error),
                )
                    .into_response(),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Formated Interal: {:#}", error),
                )
                    .into_response(),
            }
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("(Generic): {:#}", self.0),
            )
                .into_response()
        }
    }
}
