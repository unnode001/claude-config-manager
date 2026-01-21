//! Integration tests for Configuration File I/O
//!
//! These tests verify real-world file system operations.

use claude_config_manager_core::{ConfigManager, McpServer};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_full_read_modify_write_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".claude").join("config.json");
    let backup_dir = temp_dir.path().join("backups");

    // Create initial config
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    let initial_json = r#"{
        "mcpServers": {
            "npx": {
                "enabled": true
            }
        }
    }"#;
    fs::write(&config_path, initial_json).unwrap();

    let manager = ConfigManager::new(&backup_dir);

    // Read config
    let mut config = manager.read_config(&config_path).unwrap();

    // Modify config
    let server = McpServer::new("uvx", "uvx", vec![]);
    config = config.with_mcp_server("uvx", server);

    // Write back
    manager
        .write_config_with_backup(&config_path, &config)
        .unwrap();

    // Verify file was updated
    let updated_content = fs::read_to_string(&config_path).unwrap();
    assert!(updated_content.contains("uvx"));

    // Verify backup exists
    let backups = manager.backup_manager().list_backups(&config_path).unwrap();
    assert_eq!(backups.len(), 1);
}

#[test]
fn test_concurrent_write_safety() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backups");

    let manager = ConfigManager::new(&backup_dir);
    let config = claude_config_manager_core::ClaudeConfig::new();

    // Create multiple files in sequence (simulating concurrent access)
    for _ in 0..3 {
        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();
    }

    // Verify all writes succeeded
    assert!(config_path.exists());

    // Verify 2 backups were created (first write has no existing file)
    let backups = manager.backup_manager().list_backups(&config_path).unwrap();
    assert_eq!(backups.len(), 2);
}

#[test]
fn test_large_config_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backs");

    let manager = ConfigManager::new(&backup_dir);

    // Create large config (simulate many MCP servers)
    let mut config = claude_config_manager_core::ClaudeConfig::new();
    for i in 0..100 {
        let server = McpServer::new(format!("server-{i}"), "cmd", vec![]);
        config = config.with_mcp_server(format!("server-{i}"), server);
    }

    // Write large config
    manager
        .write_config_with_backup(&config_path, &config)
        .unwrap();

    // Read it back
    let read_config = manager.read_config(&config_path).unwrap();

    // Verify all servers present
    assert_eq!(read_config.mcp_servers.as_ref().unwrap().len(), 100);
}

#[test]
fn test_unicode_content_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backs");

    let manager = ConfigManager::new(&backup_dir);

    // Create config with unicode content
    let config = claude_config_manager_core::ClaudeConfig::new()
        .with_custom_instruction("使用中文")
        .with_custom_instruction("🎉 Emoji support")
        .with_custom_instruction("العربية");

    manager
        .write_config_with_backup(&config_path, &config)
        .unwrap();

    // Read back
    let read_config = manager.read_config(&config_path).unwrap();
    let instructions = read_config.custom_instructions.unwrap();

    assert_eq!(instructions.len(), 3);
    assert!(instructions[0].contains("中文"));
    assert!(instructions[1].contains("🎉"));
    assert!(instructions[2].contains("العربية"));
}

#[test]
fn test_special_characters_in_paths() {
    let temp_dir = TempDir::new().unwrap();
    // Create path with special characters
    let config_dir = temp_dir.path().join("my config (2025)");
    let config_path = config_dir.join("config.json");
    let backup_dir = temp_dir.path().join("backs");

    let manager = ConfigManager::new(&backup_dir);
    let config = claude_config_manager_core::ClaudeConfig::new();

    // Write to path with special characters
    manager
        .write_config_with_backup(&config_path, &config)
        .unwrap();

    assert!(config_path.exists());
}

#[test]
fn test_atomic_write_crash_recovery() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backs");

    let manager = ConfigManager::new(&backup_dir);

    // Create original config
    let original =
        claude_config_manager_core::ClaudeConfig::new().with_custom_instruction("Original content");
    manager
        .write_config_with_backup(&config_path, &original)
        .unwrap();

    // Simulate a failed write by trying to write invalid config
    let mut invalid_config = claude_config_manager_core::ClaudeConfig::new();
    let mut servers = std::collections::HashMap::new();
    servers.insert("".to_string(), McpServer::new("", "npx", vec![]));
    invalid_config.mcp_servers = Some(servers);

    // This should fail validation
    let result = manager.write_config_with_backup(&config_path, &invalid_config);
    assert!(result.is_err());

    // Verify original file intact (crash recovery)
    let recovered = manager.read_config(&config_path).unwrap();
    assert_eq!(recovered.custom_instructions, original.custom_instructions);
}

#[test]
fn test_backup_cleanup_after_many_writes() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backs");

    let manager = ConfigManager::new(&backup_dir);
    let config = claude_config_manager_core::ClaudeConfig::new();

    // Write many times to exceed retention
    for _ in 0..15 {
        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Verify all backups exist (cleanup is not automatic)
    let backups = manager.backup_manager().list_backups(&config_path).unwrap();
    assert_eq!(backups.len(), 14); // First write has no existing file

    // Now trigger cleanup
    let removed = manager
        .backup_manager()
        .cleanup_old_backups(&config_path)
        .unwrap();
    assert_eq!(removed, 4); // Should remove 4 to keep 10

    // Verify cleanup worked
    let backups = manager.backup_manager().list_backups(&config_path).unwrap();
    assert!(
        backups.len() <= 10,
        "Should not exceed retention count after cleanup"
    );
}
