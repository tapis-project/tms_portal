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

    #[error("Not found error: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized error: {0}")]
    Unauthorized(String),
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
                ServiceError::NotFound(_) => {
                    (StatusCode::NOT_FOUND, format!("{:#}", error)).into_response()
                }
                ServiceError::Unauthorized(_) => {
                    (StatusCode::UNAUTHORIZED, format!("{:#}", error)).into_response()
                }
                ServiceError::BadRequest(_) => (
                    StatusCode::BAD_REQUEST,
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
