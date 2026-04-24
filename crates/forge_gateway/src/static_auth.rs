//! Static file authentication middleware

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;

/// Middleware to handle authentication for static files
pub struct StaticAuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for StaticAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = StaticAuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(StaticAuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct StaticAuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for StaticAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Allow public access to the main page and auth-related files
            let path = req.path();

            if path == "/" || path == "/index.html" || path.starts_with("/auth/") {
                // Public access for authentication flow
                service.call(req).await
            } else {
                // For other static files, check for authentication token
                if let Some(token) = req.headers().get("Authorization") {
                    // TODO: Validate token against token manager
                    // For now, allow access if Authorization header is present
                    service.call(req).await
                } else {
                    // Redirect to login page for unauthenticated requests
                    let mut res = actix_web::HttpResponse::TemporaryRedirect();
                    res.insert_header(("Location", "/"));
                    Err(actix_web::error::ErrorUnauthorized("Authentication required"))
                }
            }
        })
    }
}