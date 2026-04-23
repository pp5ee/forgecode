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
    // Configure tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("Logging system initialized");
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
}