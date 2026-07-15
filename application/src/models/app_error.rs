use axum::http::StatusCode;
use axum::response::IntoResponse;
use tms_lib::utils::service_error::ServiceError;
use crate::models::tms_response::TmsResponse;

pub struct AppError(anyhow::Error);
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
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