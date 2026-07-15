use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Serialize;
use crate::models::tms_response::TmsResponse;

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponseBody<T>
where
    T: Serialize,
{
    pub status: String,
    pub result: Option<T>,
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
fn internal_error_response(msg: &str) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}
