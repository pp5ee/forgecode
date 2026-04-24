use super::*;
use pretty_assertions::assert_eq;
use std::time::{SystemTime, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test fixture for creating a secure token manager with default config
    fn create_token_manager_fixture() -> SecureTokenManager {
        let config = SecurityConfig {
            token_expiration_seconds: 60, // Short expiration for testing
            max_tokens_per_user: 5,
            rate_limit_per_minute: 10,
            refresh_window_percent: 20,
            require_https: false,
            enable_audit_logging: true,
        };
        SecureTokenManager::new(config)
    }

    /// Test fixture for creating a valid token request
    fn create_token_request_fixture() -> TokenRequest {
        TokenRequest {
            permissions: vec!["read".to_string(), "write".to_string()],
        }
    }

    #[test]
    fn test_token_generation() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let actual = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        );

        let expected = actual.is_ok();
        assert!(expected, "Token generation should succeed");
    }

    #[test]
    fn test_token_validation() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let (token_str, _) = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).expect("Token generation should succeed");

        let actual = setup.validate_token(&token_str);
        let expected = actual.is_ok();
        assert!(expected, "Token validation should succeed for valid token");
    }

    #[test]
    fn test_token_expiration() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let (token_str, _) = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).expect("Token generation should succeed");

        // Simulate time passing by manually expiring the token
        let mut tokens = setup.tokens.write().unwrap();
        let token = tokens.get_mut(&uuid::Uuid::parse_str(&token_str).unwrap()).unwrap();
        token.expires_at = SystemTime::now() - Duration::from_secs(1);
        drop(tokens);

        let actual = setup.validate_token(&token_str);
        let expected = matches!(actual, Err(AuthError::TokenExpired));
        assert!(expected, "Expired token should be rejected");
    }

    #[test]
    fn test_token_refresh() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let (original_token, _) = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).expect("Token generation should succeed");

        let actual = setup.refresh_token(&original_token);
        let expected = actual.is_ok();
        assert!(expected, "Token refresh should succeed");
    }

    #[test]
    fn test_token_revocation() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let (token_str, _) = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).expect("Token generation should succeed");

        let revocation_result = setup.revoke_token(&token_str);
        assert!(revocation_result.is_ok(), "Token revocation should succeed");

        let validation_result = setup.validate_token(&token_str);
        let expected = matches!(validation_result, Err(AuthError::TokenRevoked));
        assert!(expected, "Revoked token should be rejected");
    }

    #[test]
    fn test_rate_limiting() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        // Generate multiple tokens to trigger rate limiting
        for i in 0..10 {
            let result = setup.generate_token(
                request.permissions.clone(),
                Some("test-agent".to_string()),
                Some("127.0.0.1".to_string()),
            );
            assert!(result.is_ok(), "Token generation should succeed for first 10 requests");
        }

        // 11th request should be rate limited
        let actual = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        );

        let expected = matches!(actual, Err(AuthError::RateLimitExceeded));
        assert!(expected, "Rate limiting should block excessive requests");
    }

    #[test]
    fn test_empty_permissions_rejection() {
        let setup = create_token_manager_fixture();

        let actual = setup.generate_token(
            vec!["".to_string()], // Empty permission
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        );

        let expected = matches!(actual, Err(AuthError::InvalidTokenFormat));
        assert!(expected, "Empty permissions should be rejected");
    }

    #[test]
    fn test_token_cleanup() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        // Generate some tokens
        for _ in 0..3 {
            setup.generate_token(
                request.permissions.clone(),
                Some("test-agent".to_string()),
                Some("127.0.0.1".to_string()),
            ).expect("Token generation should succeed");
        }

        // Expire all tokens manually
        let mut tokens = setup.tokens.write().unwrap();
        for token in tokens.values_mut() {
            token.expires_at = SystemTime::now() - Duration::from_secs(1);
        }
        drop(tokens);

        let actual = setup.cleanup_expired_tokens();
        let expected = 3;
        assert_eq!(actual, expected, "Should cleanup all expired tokens");
    }

    #[test]
    fn test_secure_token_refresh_limits() {
        let setup = create_token_manager_fixture();
        let request = create_token_request_fixture();

        let (original_token, _) = setup.generate_token(
            request.permissions.clone(),
            Some("test-agent".to_string()),
            Some("127.0.0.1".to_string()),
        ).expect("Token generation should succeed");

        // Refresh token multiple times (up to the limit)
        let mut current_token = original_token;
        for i in 0..5 {
            let result = setup.refresh_token(&current_token);
            assert!(result.is_ok(), "Token refresh should succeed for attempt {}", i + 1);
            (current_token, _) = result.unwrap();
        }

        // 6th refresh should exceed the limit
        let actual = setup.refresh_token(&current_token);
        let expected = matches!(actual, Err(AuthError::RefreshLimitExceeded));
        assert!(expected, "Refresh limit should be enforced");
    }

    #[test]
    fn test_invalid_token_format() {
        let setup = create_token_manager_fixture();

        let actual = setup.validate_token("invalid-token-format");
        let expected = matches!(actual, Err(AuthError::InvalidTokenFormat));
        assert!(expected, "Invalid token format should be rejected");
    }

    #[test]
    fn test_nonexistent_token() {
        let setup = create_token_manager_fixture();

        let actual = setup.validate_token("00000000-0000-0000-0000-000000000000");
        let expected = matches!(actual, Err(AuthError::TokenNotFound));
        assert!(expected, "Non-existent token should be rejected");
    }
}
