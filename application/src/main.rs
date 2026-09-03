extern crate core;

mod config;
mod db;
mod routes;
mod services;
mod utils;
mod obj_model;

use anyhow::Context;
use axum::body::Body;
use crate::config::{init_db, init_logging};
use crate::routes::{delegation, login, oauth, resource, well_known};
//use axum_extra::extract::cookie::Key;
use tms_lib::utils::service_error::ServiceError::{MethodNotAllowed, NotFound};
use axum::handler::HandlerWithoutStateExt;
use axum::response::IntoResponse;
use axum::{middleware, Router};
use axum::middleware::Next;
use chrono::{TimeDelta, Utc};
use http::Request;
use log::error;
use sqlx::PgPool;
use tokio::spawn;
use tower_http::services::ServeDir;
use tower_http::trace::{TraceLayer};
use tracing::{instrument};
use url::Url;
use uuid::Uuid;
use crate::db::issued_tokens_dao::db_cleanup_tokens;
use crate::utils::app_error::AppError;
use crate::utils::configuration::Configuration;

#[derive(Debug, Clone)]
struct AppState {
    // that holds the key used to encrypt cookies
    // key: Key,
    db_pool: PgPool,
}

#[tokio::main]
#[instrument]
async fn main() {
    let database_host = std::env::var("TMS_PORTAL_DB_HOST").expect("TMS_PORTAL_DB_HOST must be set");
    let database_port = std::env::var("TMS_PORTAL_DB_PORT").unwrap_or(String::from("5432"));
    let database_name = std::env::var("TMS_PORTAL_DB_NAME").unwrap_or(String::from("tms_db"));
    let database_user = std::env::var("TMS_PORTAL_DB_USER").unwrap_or(String::from("tms"));
    let database_password =
        std::env::var("TMS_PORTAL_DB_PASSWORD").expect("TMS_PORTAL_DB_PASSWORD must be set");

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

    let config = Configuration::get(&state.db_pool).await.expect("Unable to read configuration from the database");

    init_logging(&config.runtime_config).await;
    let cleanup_db_pool = state.db_pool.clone();
    spawn(async move {
        // TODO: configuration setting
        let mut interval = tokio::time::interval(tokio::time::Duration::from_mins(15));
        println!("Begin cleanup thread");
        loop {
            interval.tick().await;
            println!("Token Cleanup");

            let result:anyhow::Result<()> = async {
                match cleanup_db_pool.begin().await {
                    Ok(mut tx) => {
                        // expired an hour ago
                        let expires_before = Utc::now() - TimeDelta::hours(1);
                        match db_cleanup_tokens(&mut tx, &expires_before).await {
                            Ok(_) => {
                                tx.commit().await.context("Unable to commit transaction for token cleanup")
                            }
                            Err(error) => Err(error)
                        }
                    }
                    Err(error) => Err(error.into())
                }
            }.await;

            if let Err(result) = result {
                error!("Failed to perform database cleanup: {}", result);
            }
        }
    });


    let port = 8080;

    println!("Server running on port {0}", &port);

    log_mdc::insert("request_id", "HELLO WORLD");

    // build our application with a single route
    let app = Router::new()
        //        .merge(auth::router().await)
        .merge(well_known::router().await)
        .merge(login::router().await)
        .merge(resource::router().await)
        .merge(oauth::router().await)
        .merge(delegation::router().await)
        .layer(middleware::from_fn( | request: Request<Body>, next: Next | async move
            {
                let request_id = Uuid::new_v4().to_string();
                log_mdc::insert("request_id", request_id.clone());
                let res = next.run(request).await;
                res
            }
        ))
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
