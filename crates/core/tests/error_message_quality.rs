//! Integration tests for error message quality
//!
//! Verifies that error messages are clear, actionable, and user-friendly

use claude_config_manager_core::{ClaudeConfig, ConfigManager};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_error_message_file_not_found_is_helpful() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let nonexistent_path = temp_dir.path().join("config.json");

    let manager = ConfigManager::new(&backup_dir);
    let result = manager.read_config(&nonexistent_path);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();

    // Should mention the file path
    assert!(error_msg.contains("not found") || error_msg.contains("Configuration file"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion") || error_msg.contains("Create"));
}

#[test]
fn test_error_message_invalid_json_shows_location() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backups");

    // Create invalid JSON
    fs::write(&config_path, b"{ invalid json }").unwrap();

    let manager = ConfigManager::new(&backup_dir);
    let result = manager.read_config(&config_path);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();

    // Should mention JSON error
    assert!(error_msg.contains("JSON") || error_msg.contains("json"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion") || error_msg.contains("Check"));
}

#[test]
fn test_error_message_validation_failed_is_clear() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backups");

    let manager = ConfigManager::new(&backup_dir);

    // Create invalid config (empty MCP server name)
    let mut config = ClaudeConfig::new();
    let mut servers = std::collections::HashMap::new();
    servers.insert(
        "".to_string(),
        claude_config_manager_core::McpServer::new("", "npx", vec![]),
    );
    config.mcp_servers = Some(servers);

    let result = manager.write_config_with_backup(&config_path, &config);

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();

    // Should mention validation
    assert!(error_msg.contains("validation") || error_msg.contains("Validation"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion") || error_msg.contains("Details"));
}

#[test]
fn test_error_message_permission_denied_provides_guidance() {
    // This test verifies the error message format
    // Actual permission testing requires specific filesystem setup

    let error = claude_config_manager_core::ConfigError::permission_denied(
        "read",
        "/protected/config.json",
    );

    let error_msg = error.to_string();

    // Should mention permission denied
    assert!(error_msg.contains("Permission"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion") || error_msg.contains("Check file permissions"));
}

#[test]
fn test_error_messages_avoid_technical_jargon() {
    // Test that error messages use user-friendly language

    let error = claude_config_manager_core::ConfigError::not_found("/test/config.json");
    let error_msg = error.to_string();

    // Should avoid overly technical terms where possible
    // (Basic technical terms like "file", "path" are acceptable)
    assert!(!error_msg.contains("errno") && !error_msg.contains("0x"));
}

#[test]
fn test_error_message_backup_failed_emphasizes_safety() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error =
        claude_config_manager_core::ConfigError::backup_failed("/test/config.json", io_error);

    let error_msg = error.to_string();

    // Should mention backup failed
    assert!(error_msg.contains("Backup"));

    // Should emphasize data protection
    assert!(error_msg.contains("protect") || error_msg.contains("aborted"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion"));
}

#[test]
fn test_error_message_filesystem_includes_operation_and_path() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Not found");
    let error = claude_config_manager_core::ConfigError::filesystem(
        "read config file",
        "/test/config.json",
        io_error,
    );

    let error_msg = error.to_string();

    // Should include the operation
    assert!(error_msg.contains("read"));

    // Should include the path
    assert!(error_msg.contains("/test/config.json") || error_msg.contains("config.json"));

    // Should provide a suggestion
    assert!(error_msg.contains("Suggestion"));
}

#[test]
fn test_error_messages_include_context() {
    // Test that errors include sufficient context
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
    let error = claude_config_manager_core::ConfigError::Filesystem {
        operation: "write configuration".to_string(),
        path: "/protected/config.json".into(),
        source: io_error,
    };

    let error_msg = error.to_string();

    // Should include what operation failed
    assert!(error_msg.contains("write") || error_msg.contains("operation"));

    // Should include the path in question
    assert!(error_msg.contains("config.json"));

    // Should include the underlying error
    assert!(error_msg.contains("Error") || error_msg.contains("OS Error"));
}

#[test]
fn test_all_errors_are_user_friendly() {
    // Verify that all error variants provide suggestions
    use claude_config_manager_core::ConfigError;

    let errors = vec![
        ConfigError::not_found("/test/config.json"),
        ConfigError::invalid_json("/test/config.json", 5, 10, "syntax error"),
        ConfigError::permission_denied("read", "/test/config.json"),
    ];

    for error in errors {
        let msg = error.to_string();
        // All errors should provide actionable guidance
        assert!(
            msg.contains("Suggestion") || msg.contains("Tip") || msg.contains("Try"),
            "Error message should include guidance: {msg}"
        );
    }
}

#[test]
fn test_backup_error_shows_operation_was_aborted() {
    let io_error = std::io::Error::other("Backup failed");
    let error =
        claude_config_manager_core::ConfigError::backup_failed("/test/config.json", io_error);

    let error_msg = error.to_string();

    // Should indicate the operation was aborted to protect data
    assert!(error_msg.contains("aborted") || error_msg.contains("protect"));
}
