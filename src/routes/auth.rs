// use crate::models::login_api::TmsResponse;
// use crate::models::oauth2_api::{OAuth2AuthorizeRequest, OAuth2AuthorizeSuccess, OAuth2Response};
// use crate::services::service_error::AppError;
// use crate::AppState;
// use anyhow::Result;
// use axum::extract::State;
// use axum::routing::post;
// use axum::{debug_handler, Form, Router};
// use reqwest::StatusCode;

// pub async fn router() -> Router<AppState> {
//     Router::new().route("/oauth2/authorize", post(authorize_handler))
// }
//
// #[debug_handler]
// pub async fn authorize_handler(
//     State(app_state): State<AppState>,
//     request: Form<OAuth2AuthorizeRequest>,
// ) -> Result<TmsResponse<OAuth2Response>, AppError> {
//     let result = OAuth2Response::Success(OAuth2AuthorizeSuccess {
//         state: Some(String::from("this is the state")),
//         code: String::from("this is the code"),
//     });
//     // let result = OAuth2Response::Error(OAuth2AuthorizeError {
//     //     error: OAuth2Error::AccessDenied,
//     //     error_description: None,
//     //     error_uri: None,
//     //     state: None,
//     // });
//     Ok(TmsResponse::builder(StatusCode::OK).entity(result).build())
// }
