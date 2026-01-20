//! Configuration merging functionality
//!
//! This module provides functionality to merge multiple Claude configurations
//! following the specification:
//! - Objects (nested structures): Deep merge
//! - Arrays: Replace (higher scope wins)
//! - Primitives: Replace (higher scope wins)

use crate::ClaudeConfig;
use std::collections::HashMap;

/// Merge two configurations
///
/// The `override_config` takes precedence over `base_config`.
/// Merge strategy:
/// - Objects: Deep merge (recursive)
/// - Arrays: Replace (override replaces base)
/// - Primitives: Replace (override replaces base)
///
/// # Arguments
/// * `base_config` - Base configuration (lower priority)
/// * `override_config` - Override configuration (higher priority)
///
/// # Returns
/// Merged configuration
///
/// # Examples
/// ```
/// use claude_config_manager_core::{ClaudeConfig, McpServer, merge_configs};
///
/// let base = ClaudeConfig::new()
///     .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
///     .with_allowed_path("~/projects/base");
///
/// let override_config = ClaudeConfig::new()
///     .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
///     .with_allowed_path("~/projects/override");
///
/// let merged = merge_configs(&base, &override_config);
///
/// // Should have both MCP servers (deep merge)
/// assert_eq!(merged.mcp_servers.unwrap().len(), 2);
///
/// // Should only have override path (replace)
/// assert_eq!(merged.allowed_paths.unwrap().len(), 1);
/// ```
pub fn merge_configs(base_config: &ClaudeConfig, override_config: &ClaudeConfig) -> ClaudeConfig {
    let mut merged = base_config.clone();

    // Merge MCP servers (deep merge)
    if let Some(override_servers) = &override_config.mcp_servers {
        let merged_servers = merged.mcp_servers.get_or_insert_with(HashMap::new);
        for (name, server) in override_servers {
            merged_servers.insert(name.clone(), server.clone());
        }
    }

    // Merge allowed paths (replace)
    if override_config.allowed_paths.is_some() {
        merged.allowed_paths = override_config.allowed_paths.clone();
    }

    // Merge skills (deep merge)
    if let Some(override_skills) = &override_config.skills {
        let merged_skills = merged.skills.get_or_insert_with(HashMap::new);
        for (name, skill) in override_skills {
            merged_skills.insert(name.clone(), skill.clone());
        }
    }

    // Merge custom instructions (replace)
    if override_config.custom_instructions.is_some() {
        merged.custom_instructions = override_config.custom_instructions.clone();
    }

    // Merge unknown fields (deep merge)
    for (key, value) in &override_config.unknown {
        merged.unknown.insert(key.clone(), value.clone());
    }

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpServer;
    use crate::types::Skill;

    // TDD Test 1: Empty configs merge to empty config
    #[test]
    fn test_merge_empty_configs() {
        let base = ClaudeConfig::new();
        let override_config = ClaudeConfig::new();

        let merged = merge_configs(&base, &override_config);

        assert!(merged.mcp_servers.is_none());
        assert!(merged.allowed_paths.is_none());
        assert!(merged.skills.is_none());
        assert!(merged.custom_instructions.is_none());
    }

    // TDD Test 2: Deep merge MCP servers
    #[test]
    fn test_deep_merge_mcp_servers() {
        let base = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]));

        let override_config = ClaudeConfig::new()
            .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]));

        let merged = merge_configs(&base, &override_config);

        assert!(merged.mcp_servers.is_some());
        let servers = merged.mcp_servers.unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("npx"));
        assert!(servers.contains_key("uvx"));
    }

    // TDD Test 3: Override replaces base MCP server with same name
    #[test]
    fn test_override_replaces_same_server() {
        let base = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec!["-y".to_string()]));

        let override_config = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]));

        let merged = merge_configs(&base, &override_config);

        let servers = merged.mcp_servers.unwrap();
        assert_eq!(servers.len(), 1);
        let npx = servers.get("npx").unwrap();
        assert_eq!(npx.args.len(), 0); // Should have override args (empty)
    }

    // TDD Test 4: Arrays replace (not merge)
    #[test]
    fn test_arrays_replace() {
        let base = ClaudeConfig::new()
            .with_allowed_path("~/projects/base")
            .with_allowed_path("~/work/base");

        let override_config = ClaudeConfig::new()
            .with_allowed_path("~/projects/override");

        let merged = merge_configs(&base, &override_config);

        assert!(merged.allowed_paths.is_some());
        let paths = merged.allowed_paths.unwrap();
        assert_eq!(paths.len(), 1); // Only override path
        assert_eq!(paths[0], "~/projects/override");
    }

    // TDD Test 5: Empty override keeps base arrays
    #[test]
    fn test_empty_override_keeps_base_arrays() {
        let base = ClaudeConfig::new()
            .with_allowed_path("~/projects");

        let override_config = ClaudeConfig::new();

        let merged = merge_configs(&base, &override_config);

        assert_eq!(merged.allowed_paths.unwrap().len(), 1);
    }

    // TDD Test 6: Deep merge skills
    #[test]
    fn test_deep_merge_skills() {
        let base = ClaudeConfig::new()
            .with_skill(
                "code-review",
                Skill {
                    name: "code-review".to_string(),
                    enabled: true,
                    parameters: None,
                },
            );

        let override_config = ClaudeConfig::new()
            .with_skill(
                "test-gen",
                Skill {
                    name: "test-gen".to_string(),
                    enabled: true,
                    parameters: None,
                },
            );

        let merged = merge_configs(&base, &override_config);

        assert!(merged.skills.is_some());
        let skills = merged.skills.unwrap();
        assert_eq!(skills.len(), 2);
        assert!(skills.contains_key("code-review"));
        assert!(skills.contains_key("test-gen"));
    }

    // TDD Test 7: Custom instructions replace
    #[test]
    fn test_custom_instructions_replace() {
        let base = ClaudeConfig::new()
            .with_custom_instruction("Base instruction 1")
            .with_custom_instruction("Base instruction 2");

        let override_config = ClaudeConfig::new()
            .with_custom_instruction("Override instruction");

        let merged = merge_configs(&base, &override_config);

        assert!(merged.custom_instructions.is_some());
        let instructions = merged.custom_instructions.unwrap();
        assert_eq!(instructions.len(), 1); // Only override instruction
        assert_eq!(instructions[0], "Override instruction");
    }

    // TDD Test 8: Unknown fields are merged
    #[test]
    fn test_unknown_fields_merged() {
        let mut base = ClaudeConfig::new();
        base.unknown.insert(
            "futureFeature1".to_string(),
            serde_json::json!({"setting": 1}),
        );

        let mut override_config = ClaudeConfig::new();
        override_config.unknown.insert(
            "futureFeature2".to_string(),
            serde_json::json!({"setting": 2}),
        );

        let merged = merge_configs(&base, &override_config);

        assert!(merged.unknown.contains_key("futureFeature1"));
        assert!(merged.unknown.contains_key("futureFeature2"));
    }

    // TDD Test 9: Complex nested merge
    #[test]
    fn test_complex_nested_merge() {
        let base = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec!["-y".to_string()]))
            .with_allowed_path("~/projects/base")
            .with_custom_instruction("Base instruction")
            .with_skill(
                "code-review",
                Skill {
                    name: "code-review".to_string(),
                    enabled: true,
                    parameters: None,
                },
            );

        let override_config = ClaudeConfig::new()
            .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
            .with_allowed_path("~/projects/override")
            .with_skill(
                "test-gen",
                Skill {
                    name: "test-gen".to_string(),
                    enabled: false,
                    parameters: None,
                },
            );

        let merged = merge_configs(&base, &override_config);

        // MCP servers: both present (deep merge)
        assert_eq!(merged.mcp_servers.unwrap().len(), 2);

        // Paths: only override (replace)
        assert_eq!(merged.allowed_paths.unwrap().len(), 1);

        // Instructions: base inherited (override didn't specify)
        assert!(merged.custom_instructions.is_some());
        let instructions = merged.custom_instructions.as_ref().unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], "Base instruction");

        // Skills: both present (deep merge)
        assert_eq!(merged.skills.unwrap().len(), 2);
    }

    // TDD Test 10: Override with all fields populated
    #[test]
    fn test_override_all_fields() {
        let base = ClaudeConfig::new()
            .with_mcp_server("npx", McpServer::new("npx", "npx", vec![]))
            .with_allowed_path("~/base")
            .with_custom_instruction("Base");

        let override_config = ClaudeConfig::new()
            .with_mcp_server("uvx", McpServer::new("uvx", "uvx", vec![]))
            .with_allowed_path("~/override")
            .with_custom_instruction("Override");

        let merged = merge_configs(&base, &override_config);

        // All override values should win
        assert_eq!(merged.mcp_servers.unwrap().len(), 2); // Both servers
        assert_eq!(merged.allowed_paths.unwrap().len(), 1); // Only override path
        let instructions = merged.custom_instructions.as_ref().unwrap();
        assert_eq!(instructions.len(), 1); // Only override instruction
        assert_eq!(instructions[0], "Override");
    }
}
