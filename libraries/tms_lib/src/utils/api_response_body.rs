use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ApiResponseBody<T>
where
    T: Serialize,
{
    pub status: String,
    pub result: Option<T>,
}