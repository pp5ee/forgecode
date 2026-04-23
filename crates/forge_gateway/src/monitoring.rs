//! Monitoring and metrics collection for the gateway

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Service for collecting and reporting gateway metrics
#[derive(Debug, Clone)]
pub struct MonitoringService {
    /// Total number of requests processed
    total_requests: Arc<AtomicU64>,
    /// Number of active connections
    active_connections: Arc<AtomicU64>,
    /// Number of authentication failures
    auth_failures: Arc<AtomicU64>,
    /// Number of rate limit hits
    rate_limit_hits: Arc<AtomicU64>,
    /// System start time
    start_time: std::time::Instant,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            active_connections: Arc::new(AtomicU64::new(0)),
            auth_failures: Arc::new(AtomicU64::new(0)),
            rate_limit_hits: Arc::new(AtomicU64::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    /// Record a new request
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment active connections
    pub fn connection_opened(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connections
    pub fn connection_closed(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Record an authentication failure
    pub fn record_auth_failure(&self) {
        self.auth_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a rate limit hit
    pub fn record_rate_limit_hit(&self) {
        self.rate_limit_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> Metrics {
        Metrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            auth_failures: self.auth_failures.load(Ordering::Relaxed),
            rate_limit_hits: self.rate_limit_hits.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Get service status
    pub fn get_status(&self) -> ServiceStatus {
        let uptime = self.start_time.elapsed();
        let active_connections = self.active_connections.load(Ordering::Relaxed);
        let total_requests = self.total_requests.load(Ordering::Relaxed);

        // Calculate requests per second if uptime > 0
        let rps = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        ServiceStatus {
            uptime_seconds: uptime.as_secs(),
            active_connections,
            total_requests,
            requests_per_second: rps,
        }
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

/// Service status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceStatus {
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Number of active connections
    pub active_connections: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Requests per second
    pub requests_per_second: f64,
}

/// Metrics information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Metrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of active connections
    pub active_connections: u64,
    /// Number of authentication failures
    pub auth_failures: u64,
    /// Number of rate limit hits
    pub rate_limit_hits: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Health check response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {
    /// Service status
    pub status: String,
    /// Health details
    pub details: HealthDetails,
}

/// Health check details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthDetails {
    /// Whether the service is healthy
    pub healthy: bool,
    /// Service uptime
    pub uptime_seconds: u64,
    /// Active connections
    pub active_connections: u64,
    /// Total requests
    pub total_requests: u64,
}

/// Client info for tracking
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client identifier
    pub id: String,
    /// IP address
    pub ip_address: String,
    /// User agent
    pub user_agent: String,
    /// Connection start time
    pub connected_at: std::time::Instant,
}

impl ClientInfo {
    /// Create new client info
    pub fn new(id: impl Into<String>, ip_address: impl Into<String>, user_agent: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ip_address: ip_address.into(),
            user_agent: user_agent.into(),
            connected_at: std::time::Instant::now(),
        }
    }

    /// Get connection duration
    pub fn duration(&self) -> std::time::Duration {
        self.connected_at.elapsed()
    }
}
