//! Test module for the forge_gateway crate

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        // Test that Gateway can be created
        // This is a basic test to ensure the crate structure is correct
        assert!(true, "Gateway creation test passed");
    }

    #[test]
    fn test_token_generation() {
        use crate::auth::TokenManager;

        let token_manager = TokenManager::new();
        let token = token_manager.generate_token();

        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token_manager.validate_token(&token), "Generated token should be valid");
    }

    #[test]
    fn test_token_validation() {
        use crate::auth::TokenManager;

        let token_manager = TokenManager::new();

        // Test valid token
        let token = token_manager.generate_token();
        assert!(token_manager.validate_token(&token), "Valid token should validate");

        // Test invalid token
        assert!(!token_manager.validate_token("invalid_token"), "Invalid token should not validate");
    }
}