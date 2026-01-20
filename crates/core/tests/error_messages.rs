//! Integration tests for error message quality
//!
//! These tests ensure that all error messages are:
//! 1. Clear and actionable
//! 2. Provide helpful suggestions
//! 3. Include relevant context (paths, line numbers, etc.)

use claude_config_manager_core::ConfigError;

#[test]
fn test_not_found_error_includes_suggestion() {
    let error = ConfigError::not_found("/nonexistent/config.json");
    let message = error.to_string();

    // Should suggest creating config
    assert!(message.contains("Suggestion:"));
    assert!(message.contains("Create") || message.contains("config file"));
}

#[test]
fn test_invalid_json_error_includes_location() {
    let error = ConfigError::invalid_json("/test/config.json", 10, 5, "Unexpected token");
    let message = error.to_string();

    // Should include line number
    assert!(message.contains("line 10"));
    assert!(message.contains("column 5"));

    // Should include the actual error
    assert!(message.contains("Unexpected token"));

    // Should provide suggestion
    assert!(message.contains("Suggestion:"));
}

#[test]
fn test_validation_error_includes_rule_and_details() {
    let error = ConfigError::validation_failed(
        "McpServersRule",
        "Server 'test' already exists",
        "Use a different server name or remove the existing server first",
    );
    let message = error.to_string();

    // Should include rule name
    assert!(message.contains("McpServersRule"));

    // Should include details
    assert!(message.contains("already exists"));

    // Should include suggestion
    assert!(message.contains("Suggestion:"));
    assert!(message.contains("Use a different server name"));
}

#[test]
fn test_backup_failed_error_emphasizes_safety() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error = ConfigError::backup_failed("/test/config.json", io_error);
    let message = error.to_string();

    // Should emphasize safety
    assert!(message.contains("Operation aborted"));
    assert!(message.contains("protect your data"));

    // Should provide actionable suggestion
    assert!(message.contains("Suggestion:"));
}

#[test]
fn test_permission_denied_error_provides_guidance() {
    let error = ConfigError::permission_denied("write", "/protected/config.json");
    let message = error.to_string();

    // Should provide clear guidance
    assert!(message.contains("Permission denied"));
    assert!(message.contains("write"));
    assert!(message.contains("/protected/config.json"));

    // Should suggest fix
    assert!(message.contains("Suggestion:"));
    assert!(message.contains("file permissions"));
}

#[test]
fn test_filesystem_error_includes_operation_and_path() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Not found");
    let error = ConfigError::filesystem("read", "/test/file.txt", io_error);
    let message = error.to_string();

    // Should include operation
    assert!(message.contains("read"));

    // Should include path
    assert!(message.contains("/test/file.txt"));

    // Should include OS error details
    assert!(message.contains("OS Error:"));
}

#[test]
fn test_mcp_server_error_includes_context() {
    let error = ConfigError::mcp_server_error(
        "test-server",
        "start",
        "Failed to connect to server",
    );
    let message = error.to_string();

    // Should include server name and operation
    assert!(message.contains("test-server"));
    assert!(message.contains("start"));

    // Should include details
    assert!(message.contains("Failed to connect"));
}

#[test]
fn test_all_errors_are_user_friendly() {
    // Test that all error variants provide helpful information
    let errors = vec![
        ConfigError::not_found("/test.json"),
        ConfigError::invalid_json("/test.json", 1, 1, "test"),
        ConfigError::validation_failed("test", "test", "test"),
        ConfigError::permission_denied("test", "/test"),
        ConfigError::mcp_server_error("test", "test", "test"),
    ];

    for error in errors {
        let message = error.to_string();

        // All errors should have non-empty messages
        assert!(!message.is_empty());

        // All errors should be readable (not too cryptic)
        assert!(message.len() > 20);

        // All errors should provide some guidance
        // (either through "Suggestion:" or clear error description)
        let has_guidance = message.contains("Suggestion:")
            || message.contains("Check")
            || message.contains("ensure")
            || message.contains("Try");

        assert!(
            has_guidance,
            "Error message should provide guidance: {}",
            message
        );
    }
}

#[test]
fn test_error_messages_avoid_technical_jargon() {
    // Ensure errors use user-friendly language
    let error = ConfigError::permission_denied("write", "/test/file.json");
    let message = error.to_string();

    // Should use "Permission denied" not technical error codes
    assert!(message.contains("Permission denied"));

    // Should not include raw error codes like "EACCES"
    assert!(!message.contains("EACCES"));
    assert!(!message.contains("0x"));
}

#[test]
fn test_error_messages_include_context() {
    // All errors should include relevant context
    let error = ConfigError::not_found("/specific/path/config.json");
    let message = error.to_string();

    // Should include the specific path
    assert!(message.contains("/specific/path/config.json"));
}
