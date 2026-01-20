//! Configuration types for Claude Code
//!
//! This module defines the structure of Claude Code configuration files
//! following the specification in contracts/claude-config-spec.md.

pub mod manager;
pub mod merge;
pub mod validation;

use crate::types::{McpServer, Skill};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claude Code configuration
///
/// This represents the complete structure of a Claude Code configuration file.
/// All fields are optional to support empty configurations and forward compatibility.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// MCP (Model Context Protocol) server configurations
    ///
    /// Maps server names to their configurations.
    #[serde(rename = "mcpServers", skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, McpServer>>,

    /// Filesystem paths that Claude Code is allowed to access
    ///
    /// List of paths (can use ~ for home directory).
    #[serde(rename = "allowedPaths", skip_serializing_if = "Option::is_none")]
    pub allowed_paths: Option<Vec<String>>,

    /// Claude Code skill configurations
    ///
    /// Maps skill names to their configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<HashMap<String, Skill>>,

    /// Custom instructions for Claude Code
    ///
    /// List of instruction strings to follow.
    #[serde(rename = "customInstructions", skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<Vec<String>>,

    /// Unknown fields for forward compatibility
    ///
    /// Any fields not recognized by the current version are preserved here.
    #[serde(flatten)]
    pub unknown: HashMap<String, serde_json::Value>,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            mcp_servers: None,
            allowed_paths: None,
            skills: None,
            custom_instructions: None,
            unknown: HashMap::new(),
        }
    }
}

impl ClaudeConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an MCP server configuration
    pub fn with_mcp_server(mut self, name: impl Into<String>, server: McpServer) -> Self {
        self.mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), server);
        self
    }

    /// Add an allowed path
    pub fn with_allowed_path(mut self, path: impl Into<String>) -> Self {
        self.allowed_paths
            .get_or_insert_with(Vec::new)
            .push(path.into());
        self
    }

    /// Add a skill configuration
    pub fn with_skill(mut self, name: impl Into<String>, skill: Skill) -> Self {
        self.skills
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), skill);
        self
    }

    /// Add a custom instruction
    pub fn with_custom_instruction(mut self, instruction: impl Into<String>) -> Self {
        self.custom_instructions
            .get_or_insert_with(Vec::new)
            .push(instruction.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TDD Test 1: Empty configuration serialization
    #[test]
    fn test_empty_config_serialization() {
        let config = ClaudeConfig::new();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }

    // TDD Test 2: Empty configuration deserialization
    #[test]
    fn test_empty_config_deserialization() {
        let json = "{}";
        let config: ClaudeConfig = serde_json::from_str(json).unwrap();
        assert!(config.mcp_servers.is_none());
        assert!(config.allowed_paths.is_none());
        assert!(config.skills.is_none());
        assert!(config.custom_instructions.is_none());
    }

    // TDD Test 3: Minimal config with single MCP server
    #[test]
    fn test_minimal_config_with_mcp_server() {
        let server = McpServer::new("npx", "npx", vec![]);
        let config = ClaudeConfig::new().with_mcp_server("npx", server);

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ClaudeConfig = serde_json::from_str(&json).unwrap();

        assert!(parsed.mcp_servers.is_some());
        assert_eq!(parsed.mcp_servers.unwrap().len(), 1);
    }

    // TDD Test 4: Full configuration
    #[test]
    fn test_full_configuration() {
        let server = McpServer::new("npx", "npx", vec!["-y".to_string()]);
        let skill = Skill {
            name: "code-review".to_string(),
            enabled: true,
            parameters: Some(serde_json::json!({"strictness": "high"})),
        };

        let config = ClaudeConfig::new()
            .with_mcp_server("npx", server)
            .with_allowed_path("~/projects")
            .with_skill("code-review", skill)
            .with_custom_instruction("Be concise");

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: ClaudeConfig = serde_json::from_str(&json).unwrap();

        assert!(parsed.mcp_servers.is_some());
        assert!(parsed.allowed_paths.is_some());
        assert!(parsed.skills.is_some());
        assert!(parsed.custom_instructions.is_some());
    }

    // TDD Test 5: Unknown fields are preserved
    #[test]
    fn test_unknown_fields_preserved() {
        let json = r#"{
            "mcpServers": {
                "npx": {
                    "enabled": true
                }
            },
            "futureFeature": {
                "someSetting": 42
            }
        }"#;

        let config: ClaudeConfig = serde_json::from_str(json).unwrap();
        assert!(config.unknown.contains_key("futureFeature"));
    }

    // TDD Test 6: Serialization format matches Claude Code
    #[test]
    fn test_serialization_format() {
        let server = McpServer::new("npx", "npx", vec![]);
        let config = ClaudeConfig::new().with_mcp_server("npx", server);

        let json = serde_json::to_string_pretty(&config).unwrap();

        // Verify it's pretty-printed with proper structure
        assert!(json.contains("mcpServers"));
        assert!(json.contains("enabled"));
    }

    // TDD Test 7: Config with custom instructions
    #[test]
    fn test_custom_instructions() {
        let config = ClaudeConfig::new()
            .with_custom_instruction("Be concise")
            .with_custom_instruction("Include examples");

        let json = serde_json::to_string(&config).unwrap();
        let parsed: ClaudeConfig = serde_json::from_str(&json).unwrap();

        assert!(parsed.custom_instructions.is_some());
        let instructions = parsed.custom_instructions.unwrap();
        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0], "Be concise");
        assert_eq!(instructions[1], "Include examples");
    }

    // TDD Test 8: Builder pattern methods work correctly
    #[test]
    fn test_builder_pattern() {
        let server = McpServer::new("test", "cmd", vec![]);
        let config = ClaudeConfig::new()
            .with_mcp_server("test", server.clone())
            .with_allowed_path("~/projects");

        assert!(config.mcp_servers.is_some());
        assert_eq!(config.mcp_servers.as_ref().unwrap().len(), 1);
        assert_eq!(
            config.mcp_servers.as_ref().unwrap().get("test").unwrap().name,
            "test"
        );

        assert!(config.allowed_paths.is_some());
        assert_eq!(config.allowed_paths.as_ref().unwrap().len(), 1);
    }
}
