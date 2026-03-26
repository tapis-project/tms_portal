extern crate core;

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

use crate::config::init_db;
use crate::routes::auth;
//use crate::routes::auth_middleware::AuthLayer;
use crate::routes::auth_middleware::AuthLayer;
use axum::extract::FromRef;
use axum::Router;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::instrument;

#[derive(Debug, Clone)]
struct AppState {
    // that holds the key used to encrypt cookies
    key: Key,
    db_pool: PgPool,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
#[instrument]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // TODO: move this to some kind of runtime params thing
    let database_url = std::env::var("TMS_DATABASE_URL").expect("TMS_DATABASE_URL must be set");

    let state = AppState {
        // Generate a secure key
        //
        // TODO:  You probably don't wanna generate a new one each time the app starts though
        key: Key::generate(),
        db_pool: init_db(&database_url).await,
    };

    println!("Running sqlx/Postgresql migration");
    sqlx::migrate!("./migrations/")
        .run(&state.db_pool)
        .await
        .unwrap();
    println!("Server running at http://localhost:8080");

    // build our application with a single route
    let app = Router::new()
        .merge(auth::router(&state).await)
        //        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(AuthLayer::new(state.clone())), //                .layer(AuthLayer::new()),
        )
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
