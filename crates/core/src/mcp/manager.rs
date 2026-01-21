//! MCP Server Manager
//!
//! This module provides functionality for managing MCP (Model Context Protocol) servers
//! in Claude Code configuration files.

use crate::{
    error::{ConfigError, Result},
    paths::get_global_config_path,
    types::{ConfigScope, McpServer},
    ConfigManager,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// MCP Server Manager
///
/// Handles CRUD operations for MCP servers in Claude Code configurations.
/// Supports both global and project-scoped server management.
#[derive(Debug, Clone)]
pub struct McpManager {
    /// Configuration manager for reading/writing configs
    config_manager: ConfigManager,
    /// Optional custom global config path (for testing)
    custom_global_config: Option<PathBuf>,
}

impl McpManager {
    /// Create a new McpManager
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store backups
    pub fn new(backup_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_manager: ConfigManager::new(backup_dir),
            custom_global_config: None,
        }
    }

    /// Create a new McpManager with a custom global config path (for testing)
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store backups
    /// * `custom_global_config` - Custom path for global config
    #[cfg(test)]
    pub fn with_custom_global_config(
        backup_dir: impl Into<PathBuf>,
        custom_global_config: impl Into<PathBuf>,
    ) -> Self {
        Self {
            config_manager: ConfigManager::new(backup_dir),
            custom_global_config: Some(custom_global_config.into()),
        }
    }

    /// List MCP servers
    ///
    /// Returns all servers configured at the specified scope.
    ///
    /// # Arguments
    /// * `scope` - Configuration scope (Global or Project)
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Returns
    /// HashMap of server name -> McpServer
    ///
    /// # Errors
    /// Returns an error if:
    /// - Config file cannot be read
    /// - JSON is invalid
    pub fn list_servers(
        &self,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<HashMap<String, McpServer>> {
        let (config, _path) = self.read_config_for_scope(scope, project_path)?;
        Ok(config.mcp_servers.unwrap_or_default())
    }

    /// Enable an MCP server
    ///
    /// Sets the `enabled` field to true for the specified server.
    ///
    /// # Arguments
    /// * `name` - Server name
    /// * `scope` - Configuration scope
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Server doesn't exist
    /// - Config file cannot be read/written
    pub fn enable_server(
        &self,
        name: &str,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<()> {
        self.set_server_enabled(name, true, scope, project_path)
    }

    /// Disable an MCP server
    ///
    /// Sets the `enabled` field to false for the specified server.
    ///
    /// # Arguments
    /// * `name` - Server name
    /// * `scope` - Configuration scope
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Server doesn't exist
    /// - Config file cannot be read/written
    pub fn disable_server(
        &self,
        name: &str,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<()> {
        self.set_server_enabled(name, false, scope, project_path)
    }

    /// Set server enabled status
    ///
    /// Internal helper to enable/disable servers.
    fn set_server_enabled(
        &self,
        name: &str,
        enabled: bool,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<()> {
        let (mut config, config_path) = self.read_config_for_scope(scope, project_path)?;

        // Check if server exists
        let servers = config.mcp_servers.as_ref().ok_or_else(|| {
            ConfigError::Generic("No MCP servers configured. Use 'add' command first.".to_string())
        })?;

        if !servers.contains_key(name) {
            return Err(ConfigError::Generic(format!(
                "MCP server '{}' not found. Available servers: {}",
                name,
                servers.keys().cloned().collect::<Vec<_>>().join(", ")
            )));
        }

        // Update enabled status
        if let Some(servers) = config.mcp_servers.as_mut() {
            if let Some(server) = servers.get_mut(name) {
                server.enabled = enabled;
            }
        }

        // Write back
        self.config_manager
            .write_config_with_backup(&config_path, &config)?;

        tracing::info!(
            "MCP server '{}' {}",
            name,
            if enabled { "enabled" } else { "disabled" }
        );

        Ok(())
    }

    /// Add a new MCP server
    ///
    /// Adds a server configuration at the specified scope.
    ///
    /// # Arguments
    /// * `name` - Server name (will be used as HashMap key)
    /// * `server` - Server configuration to add
    /// * `scope` - Configuration scope
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Server name is empty
    /// - Server with same name already exists
    /// - Config file cannot be read/written
    pub fn add_server(
        &self,
        name: &str,
        mut server: McpServer,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<()> {
        let name = name.trim();

        if name.is_empty() {
            return Err(ConfigError::validation_failed(
                "Server name cannot be empty",
                "name is empty",
                "provide a non-empty server name",
            ));
        }

        // Update server's internal name (for consistency)
        server.name = name.to_string();

        let (mut config, config_path) = self.read_config_for_scope(scope, project_path)?;

        // Initialize servers HashMap if needed
        if config.mcp_servers.is_none() {
            config.mcp_servers = Some(HashMap::new());
        }

        // Check if server already exists
        let servers = config.mcp_servers.as_mut().unwrap();
        if servers.contains_key(name) {
            return Err(ConfigError::Generic(format!(
                "MCP server '{name}' already exists. Use 'remove' command first or 'set' to modify."
            )));
        }

        // Add server (name is the key, server contains the config)
        servers.insert(name.to_string(), server);

        // Write back
        self.config_manager
            .write_config_with_backup(&config_path, &config)?;

        tracing::info!("MCP server '{}' added", name);

        Ok(())
    }

    /// Remove an MCP server
    ///
    /// Removes a server configuration from the specified scope.
    ///
    /// # Arguments
    /// * `name` - Server name to remove
    /// * `scope` - Configuration scope
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Errors
    /// Returns an error if:
    /// - Server doesn't exist
    /// - Config file cannot be read/written
    pub fn remove_server(
        &self,
        name: &str,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<()> {
        let (mut config, config_path) = self.read_config_for_scope(scope, project_path)?;

        // Check if servers exist
        let servers = config.mcp_servers.as_mut().ok_or_else(|| {
            ConfigError::Generic(format!(
                "No MCP servers configured. Cannot remove '{name}'."
            ))
        })?;

        // Check if server exists
        if !servers.contains_key(name) {
            return Err(ConfigError::Generic(format!(
                "MCP server '{}' not found. Available servers: {}",
                name,
                servers.keys().cloned().collect::<Vec<_>>().join(", ")
            )));
        }

        // Remove server
        servers.remove(name);

        // Clean up empty HashMap
        if servers.is_empty() {
            config.mcp_servers = None;
        }

        // Write back
        self.config_manager
            .write_config_with_backup(&config_path, &config)?;

        tracing::info!("MCP server '{}' removed", name);

        Ok(())
    }

    /// Get detailed information about a specific server
    ///
    /// # Arguments
    /// * `name` - Server name
    /// * `scope` - Configuration scope
    /// * `project_path` - Project path (required if scope is Project)
    ///
    /// # Returns
    /// The server configuration if found
    ///
    /// # Errors
    /// Returns an error if:
    /// - Server doesn't exist
    /// - Config file cannot be read
    pub fn get_server(
        &self,
        name: &str,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<McpServer> {
        let mut servers = self.list_servers(scope, project_path)?;

        servers.remove(name).ok_or_else(|| {
            ConfigError::Generic(format!(
                "MCP server '{}' not found. Available servers: {}",
                name,
                servers.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
        })
    }

    /// Read configuration for the specified scope
    ///
    /// Internal helper that returns both the config and its file path.
    fn read_config_for_scope(
        &self,
        scope: &ConfigScope,
        project_path: Option<&Path>,
    ) -> Result<(crate::ClaudeConfig, PathBuf)> {
        self.read_config_for_scope_with_path(scope, project_path, None)
    }

    /// Read configuration with optional custom config path (for testing)
    fn read_config_for_scope_with_path(
        &self,
        scope: &ConfigScope,
        project_path: Option<&Path>,
        custom_config_path: Option<&Path>,
    ) -> Result<(crate::ClaudeConfig, PathBuf)> {
        let config_path = if let Some(custom) = custom_config_path {
            custom.to_path_buf()
        } else {
            match scope {
                ConfigScope::Global => {
                    // Use custom global config if available (for testing), otherwise use default
                    if let Some(ref custom) = self.custom_global_config {
                        custom.clone()
                    } else {
                        get_global_config_path()
                    }
                }
                ConfigScope::Project => {
                    let path = project_path.ok_or_else(|| {
                        ConfigError::Generic("Project path required for Project scope".to_string())
                    })?;
                    path.join(".claude").join("config.json")
                }
            }
        };

        let config = if config_path.exists() {
            self.config_manager.read_config(&config_path)?
        } else {
            crate::ClaudeConfig::new()
        };

        Ok((config, config_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test McpManager with a temporary config path
    fn create_test_manager(temp_dir: &Path) -> McpManager {
        let backup_dir = temp_dir.join("backups");
        let config_path = temp_dir.join("config.json");
        McpManager::with_custom_global_config(&backup_dir, &config_path)
    }

    // TDD Test 1: List servers from empty config
    #[test]
    fn test_list_servers_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        let result = manager.list_servers(&ConfigScope::Global, None);

        assert!(result.is_ok());
        let servers = result.unwrap();
        assert_eq!(servers.len(), 0);
    }

    // TDD Test 2: Add and list server
    #[test]
    fn test_add_and_list_server() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // Add a server
        let server = McpServer::new("test-server", "npx", vec!["-y".to_string()]);
        manager
            .add_server("test-server", server, &ConfigScope::Global, None)
            .unwrap();

        // List servers
        let servers = manager.list_servers(&ConfigScope::Global, None).unwrap();

        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("test-server"));
        assert_eq!(servers["test-server"].command, Some("npx".to_string()));
    }

    // TDD Test 3: Add duplicate server fails
    #[test]
    fn test_add_duplicate_server_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // Add first server
        let server = McpServer::new("test", "npx", vec![]);
        manager
            .add_server("test", server, &ConfigScope::Global, None)
            .unwrap();

        // Try to add duplicate
        let server2 = McpServer::new("test", "uvx", vec![]);
        let result = manager.add_server("test", server2, &ConfigScope::Global, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    // TDD Test 4: Enable/disable server
    #[test]
    fn test_enable_disable_server() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // Add disabled server
        let mut server = McpServer::new("test", "npx", vec![]);
        server.enabled = false;
        manager
            .add_server("test", server, &ConfigScope::Global, None)
            .unwrap();

        // Enable server
        manager
            .enable_server("test", &ConfigScope::Global, None)
            .unwrap();

        // Check enabled
        let servers = manager.list_servers(&ConfigScope::Global, None).unwrap();
        assert!(servers["test"].enabled);

        // Disable server
        manager
            .disable_server("test", &ConfigScope::Global, None)
            .unwrap();

        // Check disabled
        let servers = manager.list_servers(&ConfigScope::Global, None).unwrap();
        assert!(!servers["test"].enabled);
    }

    // TDD Test 5: Enable non-existent server fails
    #[test]
    fn test_enable_nonexistent_server_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        let result = manager.enable_server("nonexistent", &ConfigScope::Global, None);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("No MCP servers configured"));
    }

    // TDD Test 6: Remove server
    #[test]
    fn test_remove_server() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // Add server
        let server = McpServer::new("test", "npx", vec![]);
        manager
            .add_server("test", server, &ConfigScope::Global, None)
            .unwrap();

        // Remove server
        manager
            .remove_server("test", &ConfigScope::Global, None)
            .unwrap();

        // Verify removed
        let servers = manager.list_servers(&ConfigScope::Global, None).unwrap();
        assert_eq!(servers.len(), 0);
    }

    // TDD Test 7: Remove non-existent server fails
    #[test]
    fn test_remove_nonexistent_server_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        let result = manager.remove_server("nonexistent", &ConfigScope::Global, None);

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found") || err_msg.contains("No MCP servers configured"));
    }

    // TDD Test 8: Get server details
    #[test]
    fn test_get_server() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // Add server
        let server = McpServer::new("test", "npx", vec!["-y".to_string()]);
        manager
            .add_server("test", server, &ConfigScope::Global, None)
            .unwrap();

        // Get server
        let retrieved = manager
            .get_server("test", &ConfigScope::Global, None)
            .unwrap();

        assert_eq!(retrieved.command, Some("npx".to_string()));
        assert_eq!(retrieved.args, vec!["-y".to_string()]);
    }

    // TDD Test 9: Project-scoped operations
    #[test]
    fn test_project_scoped_operations() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("myproject");
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let backup_dir = temp_dir.path().join("backups");
        let manager = McpManager::new(&backup_dir);

        // Add project-scoped server
        let server = McpServer::new("project-server", "uvx", vec![]);
        manager
            .add_server(
                "project-server",
                server,
                &ConfigScope::Project,
                Some(&project_dir),
            )
            .unwrap();

        // List project servers
        let servers = manager
            .list_servers(&ConfigScope::Project, Some(&project_dir))
            .unwrap();

        assert_eq!(servers.len(), 1);
        assert!(servers.contains_key("project-server"));
    }

    // TDD Test 10: Project scope without path fails
    #[test]
    fn test_project_scope_without_path_fails() {
        let temp_dir = TempDir::new().unwrap();
        let manager = create_test_manager(temp_dir.path());

        let result = manager.list_servers(&ConfigScope::Project, None);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Project path required"));
    }
}
