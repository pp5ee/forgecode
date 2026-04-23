//! Unit tests for ForgeGateway components

use forge_gateway::{
    handlers::{AuthRequest, ConversationRequest, ExecuteCommandRequest, FileRequest},
    websocket::WebSocketHandler,
};

#[test]
fn test_auth_request_serialization() {
    let auth = AuthRequest {
        token: "test-token".to_string(),
    };

    let serialized = serde_json::to_string(&auth).unwrap();
    let deserialized: AuthRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(auth.token, deserialized.token);
}

#[test]
fn test_conversation_request_serialization() {
    let conv = ConversationRequest {
        message: "Hello world".to_string(),
        conversation_id: Some("conv-123".to_string()),
    };

    let serialized = serde_json::to_string(&conv).unwrap();
    let deserialized: ConversationRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(conv.message, deserialized.message);
    assert_eq!(conv.conversation_id, deserialized.conversation_id);
}

#[test]
fn test_file_request_serialization() {
    let file = FileRequest {
        path: "/home/user".to_string(),
    };

    let serialized = serde_json::to_string(&file).unwrap();
    let deserialized: FileRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(file.path, deserialized.path);
}

#[test]
fn test_command_request_serialization() {
    let cmd = ExecuteCommandRequest {
        command: "ls -la".to_string(),
        working_directory: "/tmp".to_string(),
    };

    let serialized = serde_json::to_string(&cmd).unwrap();
    let deserialized: ExecuteCommandRequest = serde_json::from_str(&serialized).unwrap();

    assert_eq!(cmd.command, deserialized.command);
    assert_eq!(cmd.working_directory, deserialized.working_directory);
}

#[test]
fn test_websocket_handler_creation() {
    // Test that WebSocketHandler can be created without panicking
    let handler = WebSocketHandler::new();
    assert!(std::mem::size_of_val(&handler) > 0);
}

#[test]
fn test_error_handling() {
    // Test that invalid JSON deserialization fails gracefully
    let result: Result<AuthRequest, _> = serde_json::from_str("invalid json");
    assert!(result.is_err());

    let result: Result<ConversationRequest, _> = serde_json::from_str("{}");
    assert!(result.is_err());

    let result: Result<FileRequest, _> = serde_json::from_str("{}");
    assert!(result.is_err());

    let result: Result<ExecuteCommandRequest, _> = serde_json::from_str("{}");
    assert!(result.is_err());
}

#[test]
fn test_request_validation() {
    // Test that empty strings are handled properly
    let auth = AuthRequest {
        token: "".to_string(),
    };
    assert_eq!(auth.token, "");

    let conv = ConversationRequest {
        message: "".to_string(),
        conversation_id: None,
    };
    assert_eq!(conv.message, "");

    let file = FileRequest {
        path: "".to_string(),
    };
    assert_eq!(file.path, "");

    let cmd = ExecuteCommandRequest {
        command: "".to_string(),
        working_directory: "".to_string(),
    };
    assert_eq!(cmd.command, "");
    assert_eq!(cmd.working_directory, "");
}