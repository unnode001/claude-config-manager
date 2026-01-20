//! Shared types used throughout the core library

use serde::{Deserialize, Serialize};

/// Configuration scope (where a config applies)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    /// Global/user configuration (~/.claude/config.json)
    Global,
    /// Project-specific configuration (<project>/.claude/config.json)
    Project,
}

impl ConfigScope {
    /// Get the display name for this scope
    pub fn display_name(self) -> &'static str {
        match self {
            ConfigScope::Global => "global",
            ConfigScope::Project => "project",
        }
    }
}

/// Configuration layer (for merge operations)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigLayer {
    /// Global configuration layer
    Global,
    /// Project configuration layer
    Project(PathLayer),
}

/// Path information for project configurations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathLayer {
    /// Project root directory
    pub root: String,
    /// Relative path to .claude directory
    pub claude_dir: String,
}

/// MCP server configuration
///
/// This represents a single MCP server that can be enabled/disabled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServer {
    /// Server identifier (not serialized in JSON - the key is the name)
    #[serde(skip_deserializing)]
    pub name: String,
    /// Whether this server is enabled
    pub enabled: bool,
    /// Command to run (e.g., "npx", "uvx")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables for the server
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

impl McpServer {
    /// Create a new MCP server configuration
    pub fn new(name: impl Into<String>, command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
            command: Some(command.into()),
            args,
            env: std::collections::HashMap::new(),
        }
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Enable this server
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable this server
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

/// Skill configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    /// Skill identifier (not serialized in JSON - the key is the name)
    #[serde(skip_deserializing)]
    pub name: String,
    /// Whether this skill is enabled
    pub enabled: bool,
    /// Skill-specific parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Configuration file metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// File path
    pub path: String,
    /// Scope (global or project)
    pub scope: ConfigScope,
    /// Last modified timestamp
    pub modified: chrono::DateTime<chrono::Utc>,
}

/// Source tracking for configuration values
///
/// Tracks which configuration layer (global or project) a value came from
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    /// Map of key paths to their source scope
    pub sources: std::collections::HashMap<String, ConfigScope>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self {
            sources: std::collections::HashMap::new(),
        }
    }

    /// Add a source for a key path
    pub fn insert(&mut self, key_path: impl Into<String>, scope: ConfigScope) {
        self.sources.insert(key_path.into(), scope);
    }

    /// Get the source for a key path
    pub fn get(&self, key_path: &str) -> Option<&ConfigScope> {
        self.sources.get(key_path)
    }

    /// Check if a key path is from global scope
    pub fn is_global(&self, key_path: &str) -> bool {
        self.get(key_path) == Some(&ConfigScope::Global)
    }

    /// Check if a key path is from project scope
    pub fn is_project(&self, key_path: &str) -> bool {
        self.get(key_path) == Some(&ConfigScope::Project)
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration difference
///
/// Represents a single difference between two configurations
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigDiff {
    /// Value was added (exists in right but not in left)
    Added {
        key_path: String,
        value: serde_json::Value,
    },
    /// Value was removed (exists in left but not in right)
    Removed {
        key_path: String,
        value: serde_json::Value,
    },
    /// Value was modified (exists in both but different)
    Modified {
        key_path: String,
        old_value: serde_json::Value,
        new_value: serde_json::Value,
    },
}

impl ConfigDiff {
    /// Get the key path for this diff
    pub fn key_path(&self) -> &str {
        match self {
            ConfigDiff::Added { key_path, .. } => key_path,
            ConfigDiff::Removed { key_path, .. } => key_path,
            ConfigDiff::Modified { key_path, .. } => key_path,
        }
    }
}

/// Backup information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup file path
    pub path: String,
    /// Original file path
    pub original_path: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Backup size in bytes
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_scope_display_name() {
        assert_eq!(ConfigScope::Global.display_name(), "global");
        assert_eq!(ConfigScope::Project.display_name(), "project");
    }

    #[test]
    fn test_mcp_server_new() {
        let server = McpServer::new(
            "test",
            "npx",
            vec![
                "-y".to_string(),
                "@modelcontextprotocol/server-everything".to_string(),
            ],
        );
        assert_eq!(server.name, "test");
        assert_eq!(server.command, Some("npx".to_string()));
        assert!(server.enabled);
        assert!(server.env.is_empty());
    }

    #[test]
    fn test_mcp_server_with_env() {
        let server = McpServer::new("test", "npx", vec![]).with_env("API_KEY", "secret");

        assert_eq!(server.env.len(), 1);
        assert_eq!(server.env.get("API_KEY"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_mcp_server_enable_disable() {
        let mut server = McpServer::new("test", "npx", vec![]);
        assert!(server.enabled);

        server.disable();
        assert!(!server.enabled);

        server.enable();
        assert!(server.enabled);
    }

    #[test]
    fn test_config_layer_serialization() {
        let layer = ConfigLayer::Global;
        let json = serde_json::to_string(&layer).unwrap();
        assert_eq!(json, r#""global""#);
    }

    #[test]
    fn test_config_scope_serialization() {
        let scope = ConfigScope::Global;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, r#""global""#);

        let scope = ConfigScope::Project;
        let json = serde_json::to_string(&scope).unwrap();
        assert_eq!(json, r#""project""#);
    }
}
