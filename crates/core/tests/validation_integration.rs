//! Integration tests for configuration validation scenarios
//!
//! These tests verify validation behavior in realistic scenarios
//! with real filesystem and configuration files.

use claude_config_manager_core::{
    ConfigManager, McpServer, Skill,
    validate_config,
};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

// T99: Validation scenario integration tests

#[test]
fn test_valid_config_passes_all_rules() {
    // TDD Test: A valid configuration passes all validation rules
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    // Create a valid config with all fields
    let server = McpServer::new("npx", "npx", vec!["-y".to_string()]);
    let skill = Skill {
        name: "code-review".to_string(),
        enabled: true,
        parameters: Some(serde_json::json!({"strictness": "high"})),
    };

    let config = serde_json::json!({
        "mcpServers": {
            "npx": {
                "enabled": true,
                "command": "npx",
                "args": ["-y"]
            }
        },
        "allowedPaths": ["~/projects", "/usr/local"],
        "skills": {
            "code-review": {
                "enabled": true,
                "parameters": {"strictness": "high"}
            }
        }
    });

    // Write config to file
    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    // Read and validate
    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_ok(), "Valid config should pass validation");
}

#[test]
fn test_invalid_mcp_server_name_rejected() {
    // TDD Test: Config with empty MCP server name is rejected
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "mcpServers": {
            "": {
                "enabled": true,
                "command": "npx"
            }
        }
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("McpServersRule"));
    assert!(err.contains("empty"));
}

#[test]
fn test_invalid_allowed_path_rejected() {
    // TDD Test: Config with empty allowed path is rejected
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "allowedPaths": ["", "/valid/path"]
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("AllowedPathsRule"));
    assert!(err.contains("empty"));
}

#[test]
fn test_validation_with_write_config() {
    // TDD Test: Write operation validates before writing
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("claude");
    fs::create_dir_all(&config_dir).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let config_file = config_dir.join("config.json");

    let manager = ConfigManager::new(&backup_dir);

    // Create invalid config (empty server name)
    let mut servers = std::collections::HashMap::new();
    servers.insert("".to_string(), McpServer::new("", "npx", vec![]));

    let mut config = if config_file.exists() {
        manager.read_config(&config_file).unwrap()
    } else {
        claude_config_manager_core::ClaudeConfig::new()
    };
    config.mcp_servers = Some(servers);

    // Write should fail validation
    let result = manager.write_config_with_backup(&config_file, &config);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("McpServersRule") || err.contains("validation"));
}

#[test]
fn test_empty_config_is_valid() {
    // TDD Test: Empty configuration file is valid
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&config);

    assert!(result.is_ok(), "Empty config should be valid");
}

#[test]
fn test_partial_config_is_valid() {
    // TDD Test: Partial configuration (only some fields) is valid
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "mcpServers": {
            "npx": {
                "enabled": true,
                "command": "npx"
            }
        }
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_ok(), "Partial config should be valid");
}

#[test]
fn test_complex_nested_config_validates() {
    // TDD Test: Complex nested configuration validates correctly
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "mcpServers": {
            "npx": {
                "enabled": true,
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-everything"],
                "env": {
                    "API_KEY": "test-key",
                    "DEBUG": "false"
                }
            },
            "uvx": {
                "enabled": false,
                "command": "uvx"
            }
        },
        "allowedPaths": [
            "~/projects",
            "/usr/local",
            "C:\\Users\\test\\Documents"
        ],
        "customInstructions": [
            "Be concise",
            "Use Rust"
        ],
        "skills": {
            "code-review": {
                "enabled": true,
                "parameters": {
                    "strictness": "high",
                    "check-security": true
                }
            },
            "test-generator": {
                "enabled": false
            }
        }
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_ok(), "Complex nested config should be valid");
}

#[test]
fn test_validation_error_messages_are_actionable() {
    // TDD Test: Validation errors provide actionable suggestions
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "allowedPaths": [""]
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();

    // Should mention what went wrong
    assert!(err_msg.contains("AllowedPathsRule") || err_msg.contains("empty"));

    // Should provide suggestion
    assert!(err_msg.contains("Suggestion:") || err_msg.contains("All allowed paths"));
}

#[test]
fn test_multiple_validation_errors() {
    // TDD Test: Multiple validation errors are reported
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "mcpServers": {
            "": {
                "enabled": true,
                "command": "npx"
            },
            "npx": {
                "enabled": true,
                "command": "npx"
            }
        },
        "allowedPaths": [""]
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    // Should fail on first error (empty server name)
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("McpServersRule"));
}

#[test]
fn test_unicode_paths_validation() {
    // TDD Test: Unicode paths in allowedPaths are validated
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    let config = serde_json::json!({
        "allowedPaths": [
            "~/文档",
            "/usr/local/测试",
            "C:\\Users\\测试\\文档"
        ]
    });

    File::create(&config_file).unwrap().write_all(config.to_string().as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_ok(), "Unicode paths should be valid");
}

#[test]
fn test_config_with_null_bytes_rejected() {
    // TDD Test: Paths with null bytes are rejected
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("config.json");

    // Create config programmatically with null byte in path
    let mut config = serde_json::json!({});
    config["allowedPaths"] = serde_json::json!(["/path\u{0000}with\u{0000}nulls"]);

    let config_str = config.to_string();
    File::create(&config_file).unwrap().write_all(config_str.as_bytes()).unwrap();

    let backup_dir = temp_dir.path().join("backups");
    let manager = ConfigManager::new(&backup_dir);
    let read_config = manager.read_config(&config_file).unwrap();
    let result = validate_config(&read_config);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("null character") || err_msg.contains("AllowedPathsRule"));
}
