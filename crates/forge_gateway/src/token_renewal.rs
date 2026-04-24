use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::auth::{SecureToken, SecureTokenManager, AuthError};

/// Token renewal manager for handling automatic token refresh
pub struct TokenRenewalManager {
    /// Active renewal sessions
    renewal_sessions: Arc<RwLock<HashMap<Uuid, RenewalSession>>>,
    /// Token manager for token operations
    token_manager: Arc<SecureTokenManager>,
}

impl TokenRenewalManager {
    /// Create a new token renewal manager
    pub fn new(token_manager: Arc<SecureTokenManager>) -> Self {
        Self {
            renewal_sessions: Arc::new(RwLock::new(HashMap::new())),
            token_manager,
        }
    }

    /// Start automatic renewal for a token
    pub fn start_renewal(&self, token_id: Uuid) -> Result<(), AuthError> {
        let token = self.token_manager.get_token_by_id(&token_id)?;

        if token.is_expired() || token.is_revoked {
            return Err(AuthError::TokenExpired);
        }

        let session = RenewalSession::new(token_id, token.expires_at);
        self.renewal_sessions.write().unwrap().insert(token_id, session);

        log::info!("Started automatic renewal for token: {}", token_id);
        Ok(())
    }

    /// Stop automatic renewal for a token
    pub fn stop_renewal(&self, token_id: &Uuid) -> bool {
        self.renewal_sessions.write().unwrap().remove(token_id).is_some()
    }

    /// Check if a token has automatic renewal enabled
    pub fn is_renewal_active(&self, token_id: &Uuid) -> bool {
        self.renewal_sessions.read().unwrap().contains_key(token_id)
    }

    /// Get all active renewal sessions
    pub fn get_active_sessions(&self) -> Vec<RenewalSessionInfo> {
        self.renewal_sessions
            .read()
            .unwrap()
            .values()
            .map(|session| session.to_info())
            .collect()
    }

    /// Process token renewals (should be called periodically)
    pub fn process_renewals(&self) -> RenewalStats {
        let mut stats = RenewalStats::new();
        let mut sessions_to_remove = Vec::new();

        let now = SystemTime::now();
        let renewal_window = Duration::from_secs(300); // 5 minutes before expiration

        // Process each renewal session
        for (token_id, session) in self.renewal_sessions.read().unwrap().iter() {
            stats.total_sessions += 1;

            // Check if token is still valid
            match self.token_manager.get_token_by_id(token_id) {
                Ok(token) => {
                    if token.is_expired() || token.is_revoked {
                        sessions_to_remove.push(*token_id);
                        stats.expired_tokens += 1;
                        continue;
                    }

                    // Check if renewal is needed
                    if now > session.expires_at - renewal_window {
                        match self.token_manager.refresh_token_by_id(token_id) {
                            Ok((new_token, _)) => {
                                stats.successful_renewals += 1;
                                log::info!("Successfully renewed token: {}", token_id);

                                // Update session with new expiration
                                if let Some(session) = self.renewal_sessions.write().unwrap().get_mut(token_id) {
                                    session.expires_at = new_token.expires_at;
                                    session.last_renewal = SystemTime::now();
                                    session.renewal_count += 1;
                                }
                            }
                            Err(e) => {
                                stats.failed_renewals += 1;
                                log::warn!("Failed to renew token {}: {}", token_id, e);

                                // Remove session if renewal limit exceeded or token expired
                                if matches!(e, AuthError::RefreshLimitExceeded) || token.is_expired() {
                                    sessions_to_remove.push(*token_id);
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Token no longer exists
                    sessions_to_remove.push(*token_id);
                    stats.missing_tokens += 1;
                }
            }
        }

        // Remove expired/invalid sessions
        for token_id in sessions_to_remove {
            self.renewal_sessions.write().unwrap().remove(&token_id);
            stats.removed_sessions += 1;
        }

        stats
    }

    /// Get renewal statistics
    pub fn get_stats(&self) -> RenewalStats {
        let mut stats = RenewalStats::new();

        for session in self.renewal_sessions.read().unwrap().values() {
            stats.total_sessions += 1;
            stats.total_renewal_count += session.renewal_count;
        }

        stats
    }
}

/// Individual renewal session
#[derive(Debug, Clone)]
pub struct RenewalSession {
    /// Token ID being renewed
    pub token_id: Uuid,
    /// Current expiration time
    pub expires_at: SystemTime,
    /// Last renewal time
    pub last_renewal: SystemTime,
    /// Number of successful renewals
    pub renewal_count: u32,
    /// Session creation time
    pub created_at: SystemTime,
}

impl RenewalSession {
    /// Create a new renewal session
    pub fn new(token_id: Uuid, expires_at: SystemTime) -> Self {
        let now = SystemTime::now();
        Self {
            token_id,
            expires_at,
            last_renewal: now,
            renewal_count: 0,
            created_at: now,
        }
    }

    /// Convert to info structure
    pub fn to_info(&self) -> RenewalSessionInfo {
        RenewalSessionInfo {
            token_id: self.token_id,
            expires_at: self.expires_at,
            last_renewal: self.last_renewal,
            renewal_count: self.renewal_count,
            created_at: self.created_at,
        }
    }
}

/// Renewal session information for API responses
#[derive(Debug, Clone, serde::Serialize)]
pub struct RenewalSessionInfo {
    pub token_id: Uuid,
    pub expires_at: SystemTime,
    pub last_renewal: SystemTime,
    pub renewal_count: u32,
    pub created_at: SystemTime,
}

/// Renewal statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RenewalStats {
    pub total_sessions: usize,
    pub successful_renewals: u32,
    pub failed_renewals: u32,
    pub expired_tokens: u32,
    pub missing_tokens: u32,
    pub removed_sessions: u32,
    pub total_renewal_count: u32,
}

impl RenewalStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            total_sessions: 0,
            successful_renewals: 0,
            failed_renewals: 0,
            expired_tokens: 0,
            missing_tokens: 0,
            removed_sessions: 0,
            total_renewal_count: 0,
        }
    }
}

/// Background renewal processor
pub struct RenewalProcessor {
    renewal_manager: Arc<TokenRenewalManager>,
    interval: Duration,
}

impl RenewalProcessor {
    /// Create a new renewal processor
    pub fn new(renewal_manager: Arc<TokenRenewalManager>, interval: Duration) -> Self {
        Self {
            renewal_manager,
            interval,
        }
    }

    /// Start the renewal processor
    pub async fn start(self) {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;

            let stats = self.renewal_manager.process_renewals();

            if stats.successful_renewals > 0 || stats.failed_renewals > 0 {
                log::info!("Renewal processor stats: {:?}", stats);
            }
        }
    }
}

/// API handlers for token renewal management
pub mod handlers {
    use actix_web::{web, HttpResponse, Responder};
    use uuid::Uuid;

    use super::{TokenRenewalManager, RenewalStats};

    /// Start automatic renewal for a token
    #[actix_web::post("/renewal/start/{token_id}")]
    pub async fn start_renewal(
        token_id: web::Path<Uuid>,
        renewal_manager: web::Data<TokenRenewalManager>,
    ) -> impl Responder {
        match renewal_manager.start_renewal(*token_id) {
            Ok(()) => HttpResponse::Ok().json(serde_json::json!({
                "message": "Automatic renewal started",
                "token_id": token_id.to_string()
            })),
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Failed to start renewal: {}", e)
            })),
        }
    }

    /// Stop automatic renewal for a token
    #[actix_web::post("/renewal/stop/{token_id}")]
    pub async fn stop_renewal(
        token_id: web::Path<Uuid>,
        renewal_manager: web::Data<TokenRenewalManager>,
    ) -> impl Responder {
        if renewal_manager.stop_renewal(&token_id) {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Automatic renewal stopped",
                "token_id": token_id.to_string()
            }))
        } else {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Renewal session not found"
            }))
        }
    }

    /// Get active renewal sessions
    #[actix_web::get("/renewal/sessions")]
    pub async fn get_renewal_sessions(
        renewal_manager: web::Data<TokenRenewalManager>,
    ) -> impl Responder {
        let sessions = renewal_manager.get_active_sessions();
        HttpResponse::Ok().json(sessions)
    }

    /// Get renewal statistics
    #[actix_web::get("/renewal/stats")]
    pub async fn get_renewal_stats(
        renewal_manager: web::Data<TokenRenewalManager>,
    ) -> impl Responder {
        let stats = renewal_manager.get_stats();
        HttpResponse::Ok().json(stats)
    }

    /// Check if renewal is active for a token
    #[actix_web::get("/renewal/status/{token_id}")]
    pub async fn get_renewal_status(
        token_id: web::Path<Uuid>,
        renewal_manager: web::Data<TokenRenewalManager>,
    ) -> impl Responder {
        let is_active = renewal_manager.is_renewal_active(&token_id);
        HttpResponse::Ok().json(serde_json::json!({
            "token_id": token_id.to_string(),
            "renewal_active": is_active
        }))
    }
}