// pub async fn router() -> Router<AppState> {
// //    Router::new().route(":client_id/.well-known/jwks.json", get(jwks_handler))
// }

// #[debug_handler]
// pub async fn jwks_handler(
//     Path((client_id)): Path<String>,
//     State(app_state): State<AppState>,
// ) -> Result<TmsResponse<HashSet<IdpResponse>>, AppError> {
//     debug!("Client Id:{:?}", client_id);
//     Ok(TmsResponse::builder(StatusCode::OK)
//         .entity(Entity::Success("Success"))
//         .build())
// }
