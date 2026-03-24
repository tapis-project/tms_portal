use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};

// The layer that will be applied to routes
#[derive(Clone)]
pub struct AuthLayer {
    // In production, you'd inject a token validator or database pool
    expected_token: String,
}

impl AuthLayer {
    pub fn new(expected_token: impl Into<String>) -> Self {
        Self {
            expected_token: expected_token.into(),
        }
    }
}

impl<Svc> Layer<Svc> for AuthLayer {
    type Service = AuthMiddleware<Svc>;

    fn layer(&self, inner: Svc) -> Self::Service {
        AuthMiddleware {
            inner,
            expected_token: self.expected_token.clone(),
        }
    }
}

// The actual middleware service
#[derive(Clone)]
pub struct AuthMiddleware<Svc> {
    inner: Svc,
    expected_token: String,
}

impl<Svc> Service<Request<Body>> for AuthMiddleware<Svc>
where
    Svc: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    Svc::Future: Send,
{
    type Response = Svc::Response;
    type Error = Svc::Error;
    type Future = AuthFuture<Svc::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Extract the Authorization header
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Check if the token matches
        let is_authorized = auth_header
            .as_ref()
            .map(|token| token.strip_prefix("Bearer ").unwrap_or(token))
            .map(|token| token == self.expected_token)
            .unwrap_or(false);

        if is_authorized {
            // Token is valid, proceed to the inner service
            //            AuthFuture::Authorized(self.inner.call(req))
            AuthFuture::Authorized {
                future: (self.inner.call(req)),
            }
        } else {
            // Token is missing or invalid, return 401
            AuthFuture::Unauthorized
        }
    }
}

// Custom future to handle both authorized and unauthorized cases
pin_project! {
    #[project = AuthFutureProj]
    pub enum AuthFuture<F> {
        Authorized { #[pin] future: F },
        Unauthorized,
    }
}

impl<F, E> Future for AuthFuture<F>
where
    F: Future<Output = Result<Response<Body>, E>>,
{
    type Output = Result<Response<Body>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            AuthFutureProj::Authorized { future } => future.poll(cx),
            AuthFutureProj::Unauthorized => {
                let response = (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
                Poll::Ready(Ok(response))
            }
        }
    }
}
