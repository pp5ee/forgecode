use forge_gateway::auth::{SecureToken, SecureTokenManager, AuthError};
use forge_gateway::token_renewal::{RenewalSession, RenewalStats};
use std::time::{SystemTime, Duration};
use uuid::Uuid;

/// Test SecureToken creation and validation
#[test]
fn test_secure_token_creation() {
    let permissions = vec!["read".to_string(), "execute".to_string()];
    let expires_in = 3600;

    let token = SecureToken::new(permissions.clone(), expires_in);

    assert!(!token.token.is_empty());
    assert_eq!(token.permissions, permissions);
    assert!(!token.is_expired());
    assert!(!token.is_revoked);
    assert!(token.created_at <= SystemTime::now());
    assert!(token.expires_at > SystemTime::now());
}

/// Test token expiration
#[test]
fn test_token_expiration() {
    let token = SecureToken::new(vec!["read".to_string()], 1); // 1 second expiration

    // Token should not be expired immediately
    assert!(!token.is_expired());

    // Wait for expiration (simulate with future time)
    let expired_token = SecureToken {
        expires_at: SystemTime::now() - Duration::from_secs(10),
        ..token
    };

    assert!(expired_token.is_expired());
}

/// Test SecureTokenManager operations
#[test]
fn test_token_manager_operations() {
    let manager = SecureTokenManager::new();

    // Generate a token
    let token = manager.generate_token(
        vec!["read".to_string(), "write".to_string()],
        3600
    ).unwrap();

    // Validate the token
    let validated_token = manager.validate_token(&token.token).unwrap();
    assert_eq!(validated_token.id, token.id);
    assert_eq!(validated_token.permissions, token.permissions);

    // Refresh the token
    let (refreshed_token, old_token) = manager.refresh_token_by_id(&token.id).unwrap();
    assert_eq!(old_token.id, token.id);
    assert_ne!(refreshed_token.token, token.token);

    // Revoke the token
    manager.revoke_token_by_id(&token.id).unwrap();

    // Validate that revoked token fails
    let result = manager.validate_token(&token.token);
    assert!(matches!(result, Err(AuthError::TokenRevoked)));
}

/// Test token manager error cases
#[test]
fn test_token_manager_errors() {
    let manager = SecureTokenManager::new();

    // Test invalid token
    let result = manager.validate_token("invalid_token");
    assert!(matches!(result, Err(AuthError::InvalidToken)));

    // Test non-existent token ID
    let fake_id = Uuid::new_v4();
    let result = manager.refresh_token_by_id(&fake_id);
    assert!(matches!(result, Err(AuthError::TokenNotFound)));

    let result = manager.revoke_token_by_id(&fake_id);
    assert!(matches!(result, Err(AuthError::TokenNotFound)));
}

/// Test RenewalSession functionality
#[test]
fn test_renewal_session() {
    let token_id = Uuid::new_v4();
    let expires_at = SystemTime::now() + Duration::from_secs(3600);

    let session = RenewalSession::new(token_id, expires_at);

    assert_eq!(session.token_id, token_id);
    assert_eq!(session.expires_at, expires_at);
    assert_eq!(session.renewal_count, 0);
    assert!(session.created_at <= SystemTime::now());
    assert!(session.last_renewal <= SystemTime::now());
}

/// Test RenewalStats initialization
#[test]
fn test_renewal_stats() {
    let stats = RenewalStats::new();

    assert_eq!(stats.total_sessions, 0);
    assert_eq!(stats.successful_renewals, 0);
    assert_eq!(stats.failed_renewals, 0);
    assert_eq!(stats.expired_tokens, 0);
    assert_eq!(stats.missing_tokens, 0);
    assert_eq!(stats.removed_sessions, 0);
    assert_eq!(stats.total_renewal_count, 0);
}

/// Test token permission validation
#[test]
fn test_token_permissions() {
    let manager = SecureTokenManager::new();

    // Create token with specific permissions
    let token = manager.generate_token(
        vec!["read".to_string(), "execute".to_string()],
        3600
    ).unwrap();

    // Test permission checks
    assert!(token.has_permission("read"));
    assert!(token.has_permission("execute"));
    assert!(!token.has_permission("write"));
    assert!(!token.has_permission("admin"));
}

/// Test token refresh limit
#[test]
fn test_token_refresh_limit() {
    let manager = SecureTokenManager::new();

    // Generate a token
    let token = manager.generate_token(vec!["read".to_string()], 3600).unwrap();

    // Refresh multiple times (should work up to the limit)
    for _ in 0..5 {
        let result = manager.refresh_token_by_id(&token.id);
        assert!(result.is_ok());
    }

    // After hitting refresh limit, should fail
    let result = manager.refresh_token_by_id(&token.id);
    assert!(matches!(result, Err(AuthError::RefreshLimitExceeded)));
}

/// Test token serialization/deserialization
#[test]
fn test_token_serialization() {
    let token = SecureToken::new(vec!["read".to_string()], 3600);

    // Test JSON serialization
    let json = serde_json::to_string(&token).unwrap();
    let deserialized_token: SecureToken = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized_token.id, token.id);
    assert_eq!(deserialized_token.permissions, token.permissions);
    assert_eq!(deserialized_token.token, token.token);
}

/// Test token manager cleanup
#[test]
fn test_token_manager_cleanup() {
    let manager = SecureTokenManager::new();

    // Generate multiple tokens
    let token1 = manager.generate_token(vec!["read".to_string()], 1).unwrap(); // Short expiration
    let token2 = manager.generate_token(vec!["write".to_string()], 3600).unwrap();

    // Manually set token1 to expired state
    let mut tokens = manager.tokens.write().unwrap();
    if let Some(token) = tokens.get_mut(&token1.id) {
        token.expires_at = SystemTime::now() - Duration::from_secs(10);
    }
    drop(tokens);

    // Cleanup expired tokens
    manager.cleanup_expired_tokens();

    // Token1 should be removed, token2 should remain
    let result1 = manager.validate_token(&token1.token);
    let result2 = manager.validate_token(&token2.token);

    assert!(matches!(result1, Err(AuthError::TokenNotFound)));
    assert!(result2.is_ok());
}