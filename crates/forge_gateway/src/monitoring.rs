//! Monitoring and logging functionality for the ForgeCode Web Gateway

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

/// Metrics structure for tracking gateway performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayMetrics {
    /// Total number of requests processed
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Number of active WebSocket connections
    pub active_websocket_connections: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Last error message
    pub last_error: Option<String>,
    /// Timestamp of last metrics collection
    pub last_updated: Instant,
}

impl Default for GatewayMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            active_websocket_connections: 0,
            average_response_time_ms: 0.0,
            uptime_seconds: 0,
            memory_usage_bytes: 0,
            last_error: None,
            last_updated: Instant::now(),
        }
    }
}

/// Monitoring service for tracking gateway performance and health
#[derive(Debug, Clone)]
pub struct MonitoringService {
    metrics: Arc<RwLock<GatewayMetrics>>,
    start_time: Instant,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(GatewayMetrics::default())),
            start_time: Instant::now(),
        }
    }

    /// Record a successful request with its duration
    pub async fn record_successful_request(&self, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.successful_requests += 1;

        // Update average response time using weighted average
        let total_time = metrics.average_response_time_ms * metrics.total_requests as f64;
        let new_time = total_time + duration.as_millis() as f64;
        metrics.average_response_time_ms = new_time / metrics.total_requests as f64;

        metrics.last_updated = Instant::now();

        debug!("Request completed successfully in {}ms", duration.as_millis());
    }

    /// Record a failed request
    pub async fn record_failed_request(&self, error: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        metrics.last_error = Some(error.to_string());
        metrics.last_updated = Instant::now();

        warn!("Request failed: {}", error);
    }

    /// Update WebSocket connection count
    pub async fn update_websocket_connections(&self, count: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.active_websocket_connections = count;
        metrics.last_updated = Instant::now();

        info!("Active WebSocket connections: {}", count);
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> GatewayMetrics {
        let mut metrics = self.metrics.read().await.clone();

        // Update dynamic fields
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        metrics.memory_usage_bytes = self.get_memory_usage();

        metrics
    }

    /// Get memory usage (approximate)
    fn get_memory_usage(&self) -> u64 {
        // This is a simplified memory usage calculation
        // In a real implementation, you might use system-specific APIs
        std::mem::size_of::<Self>() as u64
    }

    /// Record a rate limiting event
    pub async fn record_rate_limit_event(&self, ip_address: &str, token_id: Option<&str>) {
        let client_id = token_id.unwrap_or(ip_address);
        warn!("Rate limit exceeded for client: {}", client_id);
    }

    /// Record authentication attempt
    pub async fn record_auth_attempt(&self, token_id: Option<&str>, success: bool) {
        let client_id = token_id.unwrap_or("unknown");
        if success {
            info!("Authentication successful for token: {}", client_id);
        } else {
            warn!("Authentication failed for token: {}", client_id);
        }
    }

    /// Record gateway performance metrics
    pub async fn record_performance_metrics(&self, endpoint: &str, duration: Duration, status_code: u16) {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;
        if status_code < 400 {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        // Update average response time
        let total_time = metrics.average_response_time_ms * metrics.total_requests as f64;
        let new_time = total_time + duration.as_millis() as f64;
        metrics.average_response_time_ms = new_time / metrics.total_requests as f64;

        metrics.last_updated = Instant::now();

        debug!("Endpoint '{}' processed in {}ms with status {}", endpoint, duration.as_millis(), status_code);
    }

    /// Log gateway startup
    pub fn log_startup(&self) {
        info!("ForgeCode Web Gateway starting up");
        info!("Gateway version: {}", env!("CARGO_PKG_VERSION"));
        info!("Build timestamp: {}", env!("BUILD_TIMESTAMP"));
    }

    /// Log gateway shutdown
    pub fn log_shutdown(&self) {
        info!("ForgeCode Web Gateway shutting down");
    }

    /// Log authentication event
    pub fn log_auth_event(&self, user_id: &str, success: bool) {
        if success {
            info!("Authentication successful for user: {}", user_id);
        } else {
            warn!("Authentication failed for user: {}", user_id);
        }
    }

    /// Log WebSocket event
    pub fn log_websocket_event(&self, event_type: &str, connection_id: &str) {
        debug!("WebSocket {} for connection: {}", event_type, connection_id);
    }

    /// Log file operation
    pub fn log_file_operation(&self, operation: &str, path: &str, success: bool) {
        if success {
            info!("File operation '{}' completed for path: {}", operation, path);
        } else {
            warn!("File operation '{}' failed for path: {}", operation, path);
        }
    }

    /// Log command execution
    pub fn log_command_execution(&self, command: &str, success: bool, duration: Duration) {
        if success {
            info!("Command '{}' executed successfully in {}ms", command, duration.as_millis());
        } else {
            warn!("Command '{}' failed after {}ms", command, duration.as_millis());
        }
    }
}

impl Default for MonitoringService {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub metrics: GatewayMetrics,
}

/// Initialize logging system
pub fn init_logging() {
    // Configure tracing subscriber for structured logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json() // Enable JSON formatting for structured logging
        .init();

    info!(
        event = "logging_initialized",
        version = env!("CARGO_PKG_VERSION"),
        "Logging system initialized with structured JSON format"
    );
}

/// Structured logging macros for consistent logging
#[macro_export]
macro_rules! log_gateway_event {
    ($event:expr, $($key:expr => $value:expr),*) => {
        info!(
            event = $event,
            $($key = $value,)*
            "Gateway event: {}", $event
        )
    };
}

#[macro_export]
macro_rules! log_gateway_error {
    ($event:expr, $error:expr, $($key:expr => $value:expr),*) => {
        error!(
            event = $event,
            error = $error,
            $($key = $value,)*
            "Gateway error in {}: {}", $event, $error
        )
    };
}

#[macro_export]
macro_rules! log_gateway_warning {
    ($event:expr, $warning:expr, $($key:expr => $value:expr),*) => {
        warn!(
            event = $event,
            warning = $warning,
            $($key = $value,)*
            "Gateway warning in {}: {}", $event, $warning
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_monitoring_service() {
        let monitoring = MonitoringService::new();

        // Test successful request recording
        monitoring.record_successful_request(Duration::from_millis(100)).await;
        monitoring.record_successful_request(Duration::from_millis(200)).await;

        // Test failed request recording
        monitoring.record_failed_request("Test error").await;

        // Test WebSocket connection tracking
        monitoring.update_websocket_connections(5).await;

        // Get metrics and verify
        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.active_websocket_connections, 5);
        assert!(metrics.average_response_time_ms > 0.0);
        assert!(metrics.uptime_seconds > 0);
        assert_eq!(metrics.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_logging_functions() {
        // These are mostly smoke tests to ensure the functions don't panic
        let monitoring = MonitoringService::new();

        monitoring.log_startup();
        monitoring.log_auth_event("test_user", true);
        monitoring.log_auth_event("test_user", false);
        monitoring.log_websocket_event("connect", "conn_123");
        monitoring.log_file_operation("read", "/test/path", true);
        monitoring.log_file_operation("write", "/test/path", false);
        monitoring.log_command_execution("ls -la", true, Duration::from_millis(50));
        monitoring.log_command_execution("invalid_cmd", false, Duration::from_millis(100));
        monitoring.log_shutdown();
    }

    #[test]
    fn test_health_check_response() {
        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            metrics: GatewayMetrics::default(),
        };

        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.uptime_seconds, 3600);
    }

    #[tokio::test]
    async fn test_concurrent_metrics_access() {
        // Test that metrics can be accessed concurrently without deadlocks
        let monitoring = MonitoringService::new();
        let monitoring_clone = monitoring.clone();

        let handle1 = tokio::spawn(async move {
            for _ in 0..10 {
                monitoring.record_successful_request(Duration::from_millis(50)).await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for _ in 0..10 {
                monitoring_clone.record_failed_request("concurrent_error").await;
            }
        });

        let (result1, result2) = tokio::join!(handle1, handle2);
        assert!(result1.is_ok(), "First concurrent task should complete");
        assert!(result2.is_ok(), "Second concurrent task should complete");

        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.total_requests, 20, "Should handle concurrent requests");
    }

    #[test]
    fn test_logging_macros() {
        // Test the structured logging macros
        log_gateway_event!("test_event", "user_id" => "test_user", "action" => "login");
        log_gateway_error!("test_error", "something went wrong", "file" => "test.rs", "line" => 42);
        log_gateway_warning!("test_warning", "potential issue", "severity" => "low");

        // These should compile and run without panicking
        assert!(true, "Logging macros should work correctly");
    }

    #[tokio::test]
    async fn test_performance_metrics_accuracy() {
        let monitoring = MonitoringService::new();

        // Record requests with different durations
        monitoring.record_performance_metrics("/health", Duration::from_millis(100), 200).await;
        monitoring.record_performance_metrics("/files", Duration::from_millis(200), 200).await;
        monitoring.record_performance_metrics("/execute", Duration::from_millis(300), 500).await;

        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.successful_requests, 2); // 200 status codes
        assert_eq!(metrics.failed_requests, 1);     // 500 status code
        assert_eq!(metrics.average_response_time_ms, 200.0); // (100+200+300)/3 = 200
    }

    #[tokio::test]
    async fn test_rate_limit_logging() {
        let monitoring = MonitoringService::new();

        // Test rate limit event logging
        monitoring.record_rate_limit_event("192.168.1.1", Some("token123")).await;
        monitoring.record_rate_limit_event("10.0.0.1", None).await;

        // These should complete without errors
        assert!(true, "Rate limit logging should work");
    }

    #[test]
    fn test_metrics_serialization() {
        // Test that metrics can be serialized to JSON
        let metrics = GatewayMetrics::default();
        let json_result = serde_json::to_string(&metrics);
        assert!(json_result.is_ok(), "Metrics should be serializable to JSON");

        let json_string = json_result.unwrap();
        assert!(json_string.contains("total_requests"), "JSON should contain metrics fields");
    }
}