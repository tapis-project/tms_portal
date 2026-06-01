extern crate core;

mod config;
mod db;
mod models;
mod routes;
mod services;
mod utils;

use crate::config::init_db;
use crate::routes::{login, resource, well_known};
//use axum_extra::extract::cookie::Key;
use crate::services::service_error::AppError;
use crate::services::service_error::ServiceError::{MethodNotAllowed, NotFound};
use axum::handler::HandlerWithoutStateExt;
use axum::response::IntoResponse;
use axum::Router;
use sqlx::PgPool;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use url::Url;

#[derive(Debug, Clone)]
struct AppState {
    // that holds the key used to encrypt cookies
    // key: Key,
    db_pool: PgPool,
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
    let _database_url = Url::parse(database_url_string.as_str())
        .expect(format!("The database url {0} is not valid", &database_url_string).as_str());

    let state = AppState {
        // // Generate a secure key
        // //
        // // TODO:  You probably don't wanna generate a new one each time the app starts though
        // key: Key::generate(),
        db_pool: init_db(&database_url_string).await,
    };

    println!("Running sqlx/Postgresql migration");
    sqlx::migrate!("./migrations/")
        .run(&state.db_pool)
        .await
        .unwrap();

    let port = 8080;

    println!("Server running on port {0}", &port);

    // build our application with a single route
    let app = Router::new()
        //        .merge(auth::router().await)
        .merge(well_known::router().await)
        .merge(login::router().await)
        .merge(resource::router().await)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .fallback_service(ServeDir::new("dist/").not_found_service(not_found.into_service()))
        .method_not_allowed_fallback(method_not_allowed);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn not_found() -> impl IntoResponse {
    let app_error: AppError = NotFound(String::from("Invalid Path")).into();
    app_error.into_response()
}

async fn method_not_allowed() -> impl IntoResponse {
    let app_error: AppError = MethodNotAllowed(String::from("The method is not allowed")).into();
    app_error.into_response()
}
