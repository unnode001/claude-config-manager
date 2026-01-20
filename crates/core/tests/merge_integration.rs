//! Integration tests for configuration merging
//!
//! These tests verify real-world multi-level configuration merging scenarios.

use claude_config_manager_core::{merge_configs, ClaudeConfig, McpServer, Skill};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_global_and_project_config_merge() {
    // Simulate global config with base settings
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec!["-y".to_string()]))
        .with_allowed_path("~/projects")
        .with_custom_instruction("Follow best practices");

    // Simulate project config that extends global
    let project = ClaudeConfig::new()
        .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
        .with_allowed_path("~/projects/specific-project");

    // Merge: global + project
    let merged = merge_configs(&global, &project);

    // Should have both MCP servers (deep merge)
    assert_eq!(merged.mcp_servers.as_ref().unwrap().len(), 2);
    assert!(merged.mcp_servers.as_ref().unwrap().contains_key("npx"));
    assert!(merged.mcp_servers.as_ref().unwrap().contains_key("uvx"));

    // Should only have project path (replace)
    assert_eq!(merged.allowed_paths.as_ref().unwrap().len(), 1);
    assert_eq!(
        merged.allowed_paths.as_ref().unwrap()[0],
        "~/projects/specific-project"
    );

    // Should inherit global instruction (project didn't override)
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap()[0],
        "Follow best practices"
    );
}

#[test]
fn test_three_level_merge_global_project_session() {
    // Global: base infrastructure
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
        .with_allowed_path("~/projects")
        .with_custom_instruction("Be concise");

    // Project: project-specific settings
    let project = ClaudeConfig::new()
        .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
        .with_allowed_path("~/projects/my-project")
        .with_skill(
            "code-review",
            Skill {
                name: "code-review".to_string(),
                enabled: true,
                parameters: None,
            },
        );

    // Session: temporary override
    let session = ClaudeConfig::new()
        .with_custom_instruction("Focus on performance");

    // Merge: global + project
    let merged = merge_configs(&global, &project);
    // Then merge: (global + project) + session
    let final_merged = merge_configs(&merged, &session);

    // Should have both MCP servers
    assert_eq!(
        final_merged.mcp_servers.as_ref().unwrap().len(),
        2
    );

    // Should have project path
    assert_eq!(
        final_merged.allowed_paths.as_ref().unwrap().len(),
        1
    );

    // Should have project skill
    assert!(final_merged.skills.as_ref().unwrap().contains_key("code-review"));

    // Should have session instruction (overrides both global and project)
    assert_eq!(
        final_merged.custom_instructions.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        final_merged.custom_instructions.as_ref().unwrap()[0],
        "Focus on performance"
    );
}

#[test]
fn test_empty_project_inherits_global() {
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
        .with_allowed_path("~/projects")
        .with_custom_instruction("Be helpful");

    let project = ClaudeConfig::new(); // Empty project config

    let merged = merge_configs(&global, &project);

    // Should inherit all global settings
    assert_eq!(merged.mcp_servers.as_ref().unwrap().len(), 1);
    assert_eq!(merged.allowed_paths.as_ref().unwrap().len(), 1);
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap().len(),
        1
    );
}

#[test]
fn test_project_completely_overrides_global() {
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
        .with_allowed_path("~/projects")
        .with_custom_instruction("Be helpful");

    let project = ClaudeConfig::new()
        .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
        .with_allowed_path("~/specific")
        .with_custom_instruction("Be strict");

    let merged = merge_configs(&global, &project);

    // Should have both servers (deep merge)
    assert_eq!(merged.mcp_servers.as_ref().unwrap().len(), 2);

    // Should have project values (replace)
    assert_eq!(merged.allowed_paths.as_ref().unwrap().len(), 1);
    assert_eq!(merged.allowed_paths.as_ref().unwrap()[0], "~/specific");
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap()[0],
        "Be strict"
    );
}

#[test]
fn test_merge_with_file_io() {
    let temp_dir = TempDir::new().unwrap();
    let global_path = temp_dir.path().join("global.json");
    let project_path = temp_dir.path().join("project.json");

    // Write global config
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
        .with_allowed_path("~/projects");

    let global_json = serde_json::to_string_pretty(&global).unwrap();
    fs::write(&global_path, global_json).unwrap();

    // Write project config
    let project = ClaudeConfig::new()
        .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
        .with_allowed_path("~/projects/my-project");

    let project_json = serde_json::to_string_pretty(&project).unwrap();
    fs::write(&project_path, project_json).unwrap();

    // Read both configs
    let global_read: ClaudeConfig = serde_json::from_str(&fs::read_to_string(&global_path).unwrap()).unwrap();
    let project_read: ClaudeConfig = serde_json::from_str(&fs::read_to_string(&project_path).unwrap()).unwrap();

    // Merge
    let merged = merge_configs(&global_read, &project_read);

    // Verify merge worked correctly
    assert_eq!(merged.mcp_servers.as_ref().unwrap().len(), 2);
    assert_eq!(merged.allowed_paths.as_ref().unwrap().len(), 1);
}

#[test]
fn test_merge_preserves_unknown_fields_from_both() {
    let mut global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]));
    global.unknown.insert(
        "globalFeature".to_string(),
        serde_json::json!({"setting": "global"}),
    );

    let mut project = ClaudeConfig::new()
        .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]));
    project.unknown.insert(
        "projectFeature".to_string(),
        serde_json::json!({"setting": "project"}),
    );

    let merged = merge_configs(&global, &project);

    // Should have both unknown fields
    assert!(merged.unknown.contains_key("globalFeature"));
    assert!(merged.unknown.contains_key("projectFeature"));
}

#[test]
fn test_complex_real_world_scenario() {
    // Global: development environment setup
    let global = ClaudeConfig::new()
        .with_mcp_server("npx", McpServer::new("npx", "npx", vec!["-y".to_string()]))
        .with_mcp_server("python", McpServer::new("python", "python", vec![]))
        .with_allowed_path("~/projects")
        .with_allowed_path("~/work")
        .with_custom_instruction("Follow TypeScript best practices")
        .with_custom_instruction("Include error handling")
        .with_skill(
            "code-review",
            Skill {
                name: "code-review".to_string(),
                enabled: true,
                parameters: Some(serde_json::json!({"strictness": "medium"})),
            },
        );

    // Project: specific project overrides
    let project = ClaudeConfig::new()
        .with_mcp_server("rust", McpServer::new("rust", "rust", vec![]))
        .with_allowed_path("~/projects/my-rust-project")
        .with_custom_instruction("Use async/await where appropriate")
        .with_skill(
            "code-review",
            Skill {
                name: "code-review".to_string(),
                enabled: true,
                parameters: Some(serde_json::json!({"strictness": "high"})),
            },
        );

    // Merge
    let merged = merge_configs(&global, &project);

    // MCP servers: should have all 3 (npx, python, rust)
    assert_eq!(merged.mcp_servers.as_ref().unwrap().len(), 3);
    assert!(merged.mcp_servers.as_ref().unwrap().contains_key("npx"));
    assert!(merged.mcp_servers.as_ref().unwrap().contains_key("python"));
    assert!(merged.mcp_servers.as_ref().unwrap().contains_key("rust"));

    // Paths: only project path (replace)
    assert_eq!(merged.allowed_paths.as_ref().unwrap().len(), 1);
    assert_eq!(
        merged.allowed_paths.as_ref().unwrap()[0],
        "~/projects/my-rust-project"
    );

    // Instructions: only project instruction (replace)
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap().len(),
        1
    );
    assert_eq!(
        merged.custom_instructions.as_ref().unwrap()[0],
        "Use async/await where appropriate"
    );

    // Skills: code-review should have project parameters (deep merge override)
    assert_eq!(merged.skills.as_ref().unwrap().len(), 1);
    let code_review = merged.skills.as_ref().unwrap().get("code-review").unwrap();
    assert_eq!(
        code_review.parameters.as_ref().unwrap().get("strictness").unwrap(),
        "high"
    );
}
