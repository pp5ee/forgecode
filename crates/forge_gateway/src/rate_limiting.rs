//! Rate limiting middleware for the web gateway

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Mutex;
use std::net::SocketAddr;

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Time window in seconds
    pub window_seconds: u64,
    /// Whether to rate limit by IP address
    pub limit_by_ip: bool,
    /// Whether to rate limit by token
    pub limit_by_token: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100, // 100 requests per minute by default
            window_seconds: 60, // 1 minute window
            limit_by_ip: true,
            limit_by_token: true,
        }
    }
}

/// Rate limiting data for a single client
#[derive(Debug, Clone)]
struct RateLimitData {
    requests: Vec<Instant>,
    window_start: Instant,
}

impl RateLimitData {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            window_start: Instant::now(),
        }
    }

    /// Check if the client is rate limited
    fn is_rate_limited(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let window_duration = Duration::from_secs(config.window_seconds);

        // Reset window if it has expired
        if now.duration_since(self.window_start) > window_duration {
            self.requests.clear();
            self.window_start = now;
        }

        // Remove old requests outside the current window
        self.requests.retain(|&time| now.duration_since(time) <= window_duration);

        // Check if we've exceeded the rate limit
        if self.requests.len() >= config.max_requests as usize {
            return true;
        }

        // Add current request
        self.requests.push(now);
        false
    }
}

/// Rate limiting storage
#[derive(Debug, Clone)]
pub struct RateLimitStore {
    ip_limits: Arc<Mutex<HashMap<String, RateLimitData>>>,
    token_limits: Arc<Mutex<HashMap<String, RateLimitData>>>,
    config: RateLimitConfig,
}

impl RateLimitStore {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            ip_limits: Arc::new(Mutex::new(HashMap::new())),
            token_limits: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request should be rate limited
    pub fn check_rate_limit(&self, ip: &str, token_id: Option<&str>) -> bool {
        let mut is_limited = false;

        // Rate limit by IP address
        if self.config.limit_by_ip {
            let mut ip_limits = self.ip_limits.lock().unwrap();
            let data = ip_limits.entry(ip.to_string()).or_insert_with(RateLimitData::new);
            if data.is_rate_limited(&self.config) {
                is_limited = true;
            }
        }

        // Rate limit by token
        if self.config.limit_by_token {
            if let Some(token_id) = token_id {
                let mut token_limits = self.token_limits.lock().unwrap();
                let data = token_limits.entry(token_id.to_string()).or_insert_with(RateLimitData::new);
                if data.is_rate_limited(&self.config) {
                    is_limited = true;
                }
            }
        }

        is_limited
    }

    /// Clean up old rate limiting data
    pub fn cleanup_old_data(&self) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(self.config.window_seconds * 2); // Clean up data older than 2 windows

        // Clean IP limits
        {
            let mut ip_limits = self.ip_limits.lock().unwrap();
            ip_limits.retain(|_, data| now.duration_since(data.window_start) <= window_duration);
        }

        // Clean token limits
        {
            let mut token_limits = self.token_limits.lock().unwrap();
            token_limits.retain(|_, data| now.duration_since(data.window_start) <= window_duration);
        }
    }
}

/// Rate limiting middleware factory
pub struct RateLimitMiddleware {
    store: Arc<RateLimitStore>,
}

impl RateLimitMiddleware {
    pub fn new(store: Arc<RateLimitStore>) -> Self {
        Self { store }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimitMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service: Rc::new(service),
            store: self.store.clone(),
        }))
    }
}

/// Rate limiting middleware service
pub struct RateLimitMiddlewareService<S> {
    service: Rc<S>,
    store: Arc<RateLimitStore>,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let store = self.store.clone();

        Box::pin(async move {
            // Get client IP address
            let ip = req.connection_info().realip_remote_addr()
                .unwrap_or("unknown")
                .to_string();

            // Get token ID if available
            let token_id = req.token_id()
                .map(|id| id.to_string());

            // Check rate limit
            if store.check_rate_limit(&ip, token_id.as_deref()) {
                return Err(actix_web::error::ErrorTooManyRequests(
                    "Rate limit exceeded. Please try again later."
                ));
            }

            service.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_data() {
        let mut data = RateLimitData::new();
        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
            ..Default::default()
        };

        // First request should not be limited
        assert!(!data.is_rate_limited(&config));

        // Second request should not be limited
        assert!(!data.is_rate_limited(&config));

        // Third request should be limited
        assert!(data.is_rate_limited(&config));
    }

    #[test]
    fn test_rate_limit_store() {
        let store = RateLimitStore::new(RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
            ..Default::default()
        });

        // First two requests should not be limited
        assert!(!store.check_rate_limit("127.0.0.1", None));
        assert!(!store.check_rate_limit("127.0.0.1", None));

        // Third request should be limited
        assert!(store.check_rate_limit("127.0.0.1", None));

        // Different IP should not be limited
        assert!(!store.check_rate_limit("192.168.1.1", None));
    }
}