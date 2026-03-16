mod routes;
mod models;
mod db;
mod services;

use std::ops::Deref;
use axum::{
    Router,
};
use axum::extract::FromRef;
use axum::response::Redirect;
use axum::routing::get;
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;

// async fn set_secret(
//     jar: PrivateCookieJar,
// ) -> (PrivateCookieJar, Redirect) {
//     let updated_jar = jar.add(Cookie::new("secret", "secret-data"));
//     (updated_jar, Redirect::to("/get"))
// }
//
// async fn get_secret(jar: PrivateCookieJar) {
//     if let Some(data) = jar.get("secret") {
//         // ...
//     }
// }

// our application state
#[derive(Clone)]
struct AppState {
    // that holds the key used to encrypt cookies
    key: Key,
}

// this impl tells `PrivateCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}


#[tokio::main]
async fn main() {
    let state = AppState {
        // Generate a secure key
        //
        // TODO:  You probably don't wanna generate a new one each time the app starts though
        key: Key::generate(),
    };

    println!("Server running at http://localhost:8080");
//     let token = "eyJraWQiOiIyNDRCMjM1RjZCMjhFMzQxMDhEMTAxRUFDNzM2MkM0RSIsInR5cCI6IkpXVCIsImFsZyI6IlJTMjU2In0.eyJzdWIiOiJodHRwOi8vY2lsb2dvbi5vcmcvc2VydmVyRS91c2Vycy8yMjg5MTciLCJpZHBfbmFtZSI6IlVuaXZlcnNpdHkgb2YgVGV4YXMgYXQgQXVzdGluIiwiZXBwbiI6ImRsdjY0NEB1dGV4YXMuZWR1IiwiaXNzIjoiaHR0cHM6Ly9jaWxvZ29uLm9yZyIsImdpdmVuX25hbWUiOiJEYW4iLCJhY3IiOiJodHRwczovL2lkbS51dHN5c3RlbS5lZHUvYXV0aG5jb250ZXh0L3R3b2ZhY3RvcmJhc2ljIiwiYXVkIjoiY2lsb2dvbjovY2xpZW50X2lkLzNkMzhiNTNjOTcwOTQ4OTEzNmM5YjY4YzhmNzY5Yzk5IiwibmJmIjoxNzczNDExOTc0LCJpZHAiOiJodHRwczovL2VudGVycHJpc2UubG9naW4udXRleGFzLmVkdS9pZHAvc2hpYmJvbGV0aCIsImFmZmlsaWF0aW9uIjoic3RhZmZAdXRleGFzLmVkdTttZW1iZXJAdXRleGFzLmVkdSIsImF1dGhfdGltZSI6MTc3Mjc0NTY0NCwibmFtZSI6IkRhbiBWZXJub24iLCJleHAiOjE3NzM0MTI4NzQsImZhbWlseV9uYW1lIjoiVmVybm9uIiwiaWF0IjoxNzczNDExOTc0LCJqdGkiOiJodHRwczovL2NpbG9nb24ub3JnL29hdXRoMi9pZFRva2VuLzcxODdjNzg3ZmM0YzZlYTZmNDRhZmJhM2YxNzM4ZmIvMTc3Mjc0OTc0MDQzMyJ9.Le0sg8rm46fm4TWgBaPW914EoQwubnMYc-wPkMBSkatV3hVWCBjYw5y9N1Y2Esy4SgDCrgpZWv0VmTZZEy3izdSD5zKhC7gHWctzkEqqq-j4-gvfDy8RakLA6OoS6r02QI6lPEhcoYr19CqKz0Sv-LQckxlBSc9tEfU60UNafV9oieORNlem9WNcqSjvYLGxOzWTzX6MGjy5MxyrOb-rKTqB1VkPwnB5f5iM5QZuPZTLhBpCN_RdL6WK5dKn5KLMziDc0iPHv9Ok4CnDbaXUFGOWQgcmFwlEeas_ae8vWWKP3P8LdZXPnIjrLxCtCaNGXXITvOQIhRsOgzxJJT1mUA".to_string();
// //    let token = "eyJhbGciOiJSUzI1NiIsImtpZCI6Ik1UWU9UY1ZwUnhyVkREakpyVkpiMk5HUTA4RUhsRFVpRzlfYUNOa1VJemMiLCJ0eXAiOiJKV1QifQ.eyJqdGkiOiIwZTY2MmMwZS03YzllLTQ1YjktOTBlZC01OGM5YzQzZDVkNmQiLCJpc3MiOiJodHRwczovL2Rldi5kZXZlbG9wLnRhcGlzLmlvL3YzL3Rva2VucyIsInN1YiI6InRlc3R1c2VyMkBkZXYiLCJ0YXBpcy90ZW5hbnRfaWQiOiJkZXYiLCJ0YXBpcy90b2tlbl90eXBlIjoiYWNjZXNzIiwidGFwaXMvZGVsZWdhdGlvbiI6ZmFsc2UsInRhcGlzL2RlbGVnYXRpb25fc3ViIjpudWxsLCJ0YXBpcy91c2VybmFtZSI6InRlc3R1c2VyMiIsInRhcGlzL2FjY291bnRfdHlwZSI6InVzZXIiLCJleHAiOjE3NzI2NTg1NzcsInRhcGlzL2NsaWVudF9pZCI6bnVsbCwidGFwaXMvZ3JhbnRfdHlwZSI6InBhc3N3b3JkIn0.NkTEt-TOqpXprbwCqsdwEKIaji33RcNr-oZ4qQz59v418LXYiWmhLSgINnxpaXgGwhhHdtAutdhFBtHKr2QwfnEyDGrQaR1U9bR69blBBZZtDEGoo7zvFf7IxsMlqo0pdM0OXCP8kXv9xIkoAvQV7SXlnXbGJmEtK4okX4EoVIg0M6Azkodll103SobAP2-xO0TJfSO3Raa3llGXyNDK6Xj0JmeTZLOYqc4FWrZql8OfEY7mWhYpJTsB2Gxm_anM9a09XjPo7ko__RtDe7-FkIZDplQjgSjlBkLIEb_F7XU2c6OJVtJWEu-uWxwixUsAEIqH41uL_YN5TFkbjjnljQ".to_string();
//     decode_jwt(&token).await;
//     let state = AppState {
//         // Generate a secure key
//         //
//         // You probably don't wanna generate a new one each time the app starts though
//         key: Key::generate(),
//     };

    // build our application with a single route
    let app = Router::new()
        .route("/get", get(get_secret))
        .route("/set", get(set_secret))
        .with_state(state)
        .merge(routes::auth::router());
    
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn set_secret(
    jar: PrivateCookieJar,
) -> (PrivateCookieJar, Redirect) {
    let updated_jar = jar.add(Cookie::new("secret", "secret-data"));
    (updated_jar, Redirect::to("/get"))
}

async fn get_secret(jar: PrivateCookieJar) -> String{
    if let Some(data) = jar.get("secret") {
        return "secret-data".to_owned();
    } else {
        return "Nothing found".to_owned();;
    }
}