use crate::AppState;
use axum::body::Body;
use axum::http::Request;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct Auth<S> {
    inner: S,
    state: AppState,
}

impl<S> Auth<S> {
    pub fn new(inner: S, state: AppState) -> Self {
        Auth { inner, state }
    }
}

impl<S> Service<Request<Body>> for Auth<S>
where
    S: Service<Request<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = AuthResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let auth_header_value = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let auth_string = auth_header_value
            .as_ref()
            .map(|value| value.as_str().strip_prefix("Basic ").unwrap_or(value));

        if let Some(auth_string) = auth_string {
            println!("Found Auth: {0}", &auth_string);
            let decoded_bytes = BASE64_STANDARD.decode(auth_string).unwrap();
            let decoded = String::from_utf8(decoded_bytes).unwrap();
            let (id, secret) = decoded.split_once(':').unwrap();
            println!("Decoded Auth: {0}", &decoded,);
            println!("Decoded Auth: Id:{0} Secret:{1}", &id, &secret);
            // let _f = basic_auth_is_authorized(
            //     &self.state.db_pool,
            //     &String::from(id),
            //     &String::from(secret),
            // );
        }
        AuthResponseFuture {
            response_future: self.inner.call(req),
            //                auth_future: f,
        }
    }
}

#[pin_project]
pub struct AuthResponseFuture<F> {
    #[pin]
    response_future: F,
}

impl<F, Response, Error> Future for AuthResponseFuture<F>
where
    F: Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let response_future: Pin<&mut F> = this.response_future;
        response_future.poll(cx)
    }
}

#[derive(Debug, Clone)]
pub struct AuthLayer {
    state: AppState,
}
impl AuthLayer {
    pub fn new(state: AppState) -> Self {
        AuthLayer { state }
    }
}
impl<S> Layer<S> for AuthLayer {
    type Service = Auth<S>;

    fn layer(&self, inner: S) -> Self::Service {
        Auth {
            inner,
            state: self.state.clone(),
        }
    }
}
