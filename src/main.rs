extern crate core;

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

use crate::config::init_db;
use crate::routes::auth;
use axum::extract::FromRef;
use axum::Router;
use axum_extra::extract::cookie::Key;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use url::Url;

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

    let database_host = std::env::var("TMS_AUTH_DB_HOST").expect("TMS_AUTH_DB_HOST must be set");
    let database_port = std::env::var("TMS_AUTH_DB_PORT").unwrap_or(String::from("5432"));
    let database_name = std::env::var("TMS_AUTH_DB_NAME").unwrap_or(String::from("tms_auth_db"));
    let database_user = std::env::var("TMS_AUTH_DB_USER").unwrap_or(String::from("tms_auth_user"));
    let database_password =
        std::env::var("TMS_AUTH_DB_PASSWORD").expect("TMS_AUTH_DB_PASSWORD must be set");

    let database_url_string = format!(
        "postgres://{0}:{1}@{2}:{3}/{4}",
        &database_user, &database_password, &database_host, &database_port, &database_name
    );

    // just parsing the db url to determine if it seems valid.  We actually use database_url_string.
    let database_url = Url::parse(database_url_string.as_str())
        .expect(format!("The database url {0} is not valid", &database_url_string).as_str());

    let state = AppState {
        // Generate a secure key
        //
        // TODO:  You probably don't wanna generate a new one each time the app starts though
        key: Key::generate(),
        db_pool: init_db(&database_url_string).await,
    };

    println!("Running sqlx/Postgresql migration");
    sqlx::migrate!("./migrations/")
        .run(&state.db_pool)
        .await
        .unwrap();
    println!("Server running at http://localhost:8080");

    // build our application with a single route
    let app = Router::new()
        .merge(auth::router().await)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
