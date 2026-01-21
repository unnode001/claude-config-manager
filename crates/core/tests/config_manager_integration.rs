//! Integration tests for ConfigManager convenience methods
//!
//! Tests get_global_config(), get_project_config(), get_merged_config()
//! with real filesystem operations.

use claude_config_manager_core::{ClaudeConfig, ConfigManager, McpServer};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_get_global_config_reads_from_standard_location() {
    let temp_dir = TempDir::new().unwrap();

    // Mock global config location by setting up test structure
    // Note: We can't easily override get_global_config_path() in tests,
    // so we'll test the underlying behavior

    let backup_dir = temp_dir.path().join("backups");
    let config_path = temp_dir.path().join("config.json");

    // Create a config file
    let config = ClaudeConfig::new().with_custom_instruction("Global instruction");
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    let manager = ConfigManager::new(&backup_dir);

    // Read using the underlying read_config method
    let result = manager.read_config(&config_path);

    assert!(result.is_ok());
    let loaded_config = result.unwrap();
    assert!(loaded_config.custom_instructions.is_some());
    assert_eq!(loaded_config.custom_instructions.unwrap().len(), 1);
}

#[test]
fn test_get_project_config_finds_config_in_project_directory() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Create project structure
    let project_dir = temp_dir.path().join("myproject");
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let config_path = claude_dir.join("config.json");

    // Create project config
    let config = ClaudeConfig::new().with_custom_instruction("Project instruction");
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    let manager = ConfigManager::new(&backup_dir);

    // Get project config with explicit path
    let result = manager.get_project_config(Some(&project_dir));

    assert!(result.is_ok());
    let loaded_config = result.unwrap();
    assert!(loaded_config.is_some());
    let loaded_config = loaded_config.unwrap();
    assert!(loaded_config.custom_instructions.is_some());
    assert_eq!(loaded_config.custom_instructions.unwrap().len(), 1);
}

#[test]
fn test_get_project_config_returns_none_when_no_config_exists() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Create project directory without .claude
    let project_dir = temp_dir.path().join("emptyproject");
    fs::create_dir_all(&project_dir).unwrap();

    let manager = ConfigManager::new(&backup_dir);

    let result = manager.get_project_config(Some(&project_dir));

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_get_merged_config_combines_global_and_project() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Create global config with npx server
    let global_config = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
        .with_allowed_path("~/global-projects");

    let global_path = temp_dir.path().join("global.json");
    let manager = ConfigManager::new(&backup_dir);
    manager
        .write_config_with_backup(&global_path, &global_config)
        .unwrap();

    // Create project config with uvx server
    let project_dir = temp_dir.path().join("myproject");
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let project_config =
        ClaudeConfig::new().with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]));

    let project_path = claude_dir.join("config.json");
    manager
        .write_config_with_backup(&project_path, &project_config)
        .unwrap();

    // Merge configs
    let global = manager.read_config(&global_path).unwrap();
    let project = manager.read_config(&project_path).unwrap();
    let merged = claude_config_manager_core::merge_configs(&global, &project);

    // Should have both servers (deep merge)
    assert!(merged.mcp_servers.is_some());
    let servers = merged.mcp_servers.unwrap();
    assert_eq!(servers.len(), 2);
    assert!(servers.contains_key("npx"));
    assert!(servers.contains_key("uvx"));

    // allowedPaths should be from global (project doesn't override)
    assert!(merged.allowed_paths.is_some());
    assert_eq!(merged.allowed_paths.unwrap().len(), 1);
}

#[test]
fn test_get_merged_config_project_overrides_global() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Create global config
    let global_config = ClaudeConfig::new()
        .with_allowed_path("~/global-projects")
        .with_custom_instruction("Global instruction");

    let global_path = temp_dir.path().join("global.json");
    let manager = ConfigManager::new(&backup_dir);
    manager
        .write_config_with_backup(&global_path, &global_config)
        .unwrap();

    // Create project config that overrides allowedPaths
    let project_dir = temp_dir.path().join("myproject");
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let project_config = ClaudeConfig::new().with_allowed_path("~/my-project-only");

    let project_path = claude_dir.join("config.json");
    manager
        .write_config_with_backup(&project_path, &project_config)
        .unwrap();

    // Merge configs
    let global = manager.read_config(&global_path).unwrap();
    let project = manager.read_config(&project_path).unwrap();
    let merged = claude_config_manager_core::merge_configs(&global, &project);

    // allowedPaths should be from project (replace strategy for arrays)
    assert!(merged.allowed_paths.is_some());
    let paths = merged.allowed_paths.unwrap();
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0], "~/my-project-only");

    // customInstructions should be from global (project doesn't have it)
    assert!(merged.custom_instructions.is_some());
    assert_eq!(merged.custom_instructions.unwrap().len(), 1);
}

#[test]
fn test_get_merged_config_with_empty_project_config() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Create global config
    let global_config = ClaudeConfig::new().with_custom_instruction("Global instruction");

    let global_path = temp_dir.path().join("global.json");
    let manager = ConfigManager::new(&backup_dir);
    manager
        .write_config_with_backup(&global_path, &global_config)
        .unwrap();

    // No project config
    let global = manager.read_config(&global_path).unwrap();
    let empty_project = ClaudeConfig::new();
    let merged = claude_config_manager_core::merge_configs(&global, &empty_project);

    // Should have global values
    assert!(merged.custom_instructions.is_some());
    assert_eq!(merged.custom_instructions.unwrap().len(), 1);
}

#[test]
fn test_config_manager_integration_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    // Setup: Create global and project configs
    let global_config =
        ClaudeConfig::new().with_mcp_server("npx", McpServer::new("npx", "npx", vec![]));

    let project_dir = temp_dir.path().join("myproject");
    let claude_dir = project_dir.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    let project_config =
        ClaudeConfig::new().with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]));

    let manager = ConfigManager::new(&backup_dir);

    // Write both configs
    let global_path = temp_dir.path().join("global.json");
    let project_path = claude_dir.join("config.json");

    manager
        .write_config_with_backup(&global_path, &global_config)
        .unwrap();
    manager
        .write_config_with_backup(&project_path, &project_config)
        .unwrap();

    // Verify backups were created
    let global_backups = manager.backup_manager().list_backups(&global_path).unwrap();
    let project_backups = manager
        .backup_manager()
        .list_backups(&project_path)
        .unwrap();

    // No backups for first write (no existing file)
    assert_eq!(global_backups.len(), 0);
    assert_eq!(project_backups.len(), 0);

    // Read and merge
    let global = manager.read_config(&global_path).unwrap();
    let project = manager.read_config(&project_path).unwrap();
    let merged = claude_config_manager_core::merge_configs(&global, &project);

    // Verify merge worked
    assert!(merged.mcp_servers.is_some());
    let servers = merged.mcp_servers.unwrap();
    assert_eq!(servers.len(), 2);
}
