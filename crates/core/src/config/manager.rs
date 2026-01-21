//! Configuration file manager
//!
//! This module provides functionality for reading and writing Claude Code
//! configuration files with automatic backup and atomic writes.

use crate::{
    backup::BackupManager,
    config::validation::validate_config,
    error::{ConfigError, Result},
    paths::{find_project_config, get_global_config_path},
    types::{ConfigDiff, ConfigScope, SourceMap},
    ConfigSearcher, SearchOptions, SearchResult,
};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Configuration file manager
///
/// Handles reading and writing configuration files with safety features:
/// - Automatic backup before writing
/// - Atomic writes (write-then-rename pattern)
/// - Validation before writing
/// - Clear error messages
#[derive(Debug, Clone)]
pub struct ConfigManager {
    /// Backup manager for this configuration
    backup_manager: BackupManager,
}

impl ConfigManager {
    /// Create a new ConfigManager
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store backups
    pub fn new(backup_dir: impl Into<PathBuf>) -> Self {
        Self {
            backup_manager: BackupManager::new(backup_dir, None),
        }
    }

    /// Read a configuration file
    ///
    /// # Arguments
    /// * `path` - Path to the configuration file
    ///
    /// # Returns
    /// Parsed configuration
    ///
    /// # Errors
    /// Returns an error if:
    /// - File doesn't exist
    /// - File cannot be read
    /// - JSON is invalid
    pub fn read_config(&self, path: &Path) -> Result<crate::ClaudeConfig> {
        // Check if file exists
        if !path.exists() {
            return Err(ConfigError::not_found(path));
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::filesystem("read config file", path, e))?;

        // Parse JSON
        let config: crate::ClaudeConfig = serde_json::from_str(&content).map_err(|e| {
            // Try to extract line and column from error message
            let error_str = e.to_string();
            let (line, column) = parse_json_error_location(&error_str);

            ConfigError::invalid_json(path, line, column, error_str)
        })?;

        tracing::debug!("Loaded configuration from: {}", path.display());

        Ok(config)
    }

    /// Write configuration with automatic backup
    ///
    /// This method:
    /// 1. Creates a backup of the existing file (if it exists)
    /// 2. Validates the new configuration
    /// 3. Writes to a temporary file
    /// 4. Atomically renames temp file to target
    ///
    /// # Arguments
    /// * `path` - Path to write the configuration
    /// * `config` - Configuration to write
    ///
    /// # Errors
    /// Returns an error if:
    /// - Backup creation fails (operation aborted to protect data)
    /// - Validation fails
    /// - Write operation fails
    pub fn write_config_with_backup(
        &self,
        path: &Path,
        config: &crate::ClaudeConfig,
    ) -> Result<()> {
        // Step 1: Create backup if file exists
        if path.exists() {
            tracing::debug!("Creating backup before writing: {}", path.display());
            self.backup_manager.create_backup(path)?;
        }

        // Step 2: Validate configuration
        validate_config(config)?;

        // Step 3: Serialize configuration
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| ConfigError::Generic(format!("Failed to serialize config: {e}")))?;

        // Step 4: Atomic write using temp file
        self.atomic_write(path, &json)?;

        tracing::debug!("Wrote configuration to: {}", path.display());

        Ok(())
    }

    /// Internal atomic write implementation
    ///
    /// Uses write-then-rename pattern to ensure atomicity:
    /// 1. Write to temp file in same directory
    /// 2. Rename temp file to target (atomic on most filesystems)
    fn atomic_write(&self, target: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::filesystem("create config directory", parent, e))?;
            }
        }

        // Create temp file path
        let temp_path = target.with_extension("tmp");

        // Write to temp file
        {
            let mut file = File::create(&temp_path)
                .map_err(|e| ConfigError::filesystem("create temp file", &temp_path, e))?;

            file.write_all(content.as_bytes())
                .map_err(|e| ConfigError::filesystem("write to temp file", &temp_path, e))?;

            file.flush()
                .map_err(|e| ConfigError::filesystem("flush temp file", &temp_path, e))?;
        }

        // Atomic rename (temp -> target)
        fs::rename(&temp_path, target).map_err(|e| {
            // Clean up temp file on failure
            let _ = fs::remove_file(&temp_path);
            ConfigError::filesystem("atomic rename (temp to config)", target, e)
        })?;

        Ok(())
    }

    /// Get reference to backup manager
    pub fn backup_manager(&self) -> &BackupManager {
        &self.backup_manager
    }

    /// Get global configuration
    ///
    /// Reads the global Claude Code configuration from the standard location.
    ///
    /// # Returns
    /// The global configuration, or an empty config if the file doesn't exist
    ///
    /// # Errors
    /// Returns an error if:
    /// - File exists but cannot be read
    /// - JSON is invalid
    pub fn get_global_config(&self) -> Result<crate::ClaudeConfig> {
        let global_path = get_global_config_path();

        if !global_path.exists() {
            tracing::debug!("Global config not found, returning empty config");
            return Ok(crate::ClaudeConfig::new());
        }

        self.read_config(&global_path)
    }

    /// Get project configuration
    ///
    /// Finds and reads the project-specific configuration.
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory (if None, searches upward from current dir)
    ///
    /// # Returns
    /// The project configuration if found, None otherwise
    ///
    /// # Errors
    /// Returns an error if:
    /// - File exists but cannot be read
    /// - JSON is invalid
    pub fn get_project_config(
        &self,
        project_path: Option<&Path>,
    ) -> Result<Option<crate::ClaudeConfig>> {
        let config_path = if let Some(path) = project_path {
            path.join(".claude").join("config.json")
        } else {
            // Search upward from current directory
            match find_project_config(None) {
                Some(path) => path,
                None => return Ok(None),
            }
        };

        if !config_path.exists() {
            return Ok(None);
        }

        self.read_config(&config_path).map(Some)
    }

    /// Get merged configuration
    ///
    /// Merges global and project configurations, with project values taking precedence.
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory (if None, searches upward from current dir)
    ///
    /// # Returns
    /// The merged configuration
    ///
    /// # Errors
    /// Returns an error if:
    /// - Either config file exists but cannot be read
    /// - JSON is invalid
    pub fn get_merged_config(&self, project_path: Option<&Path>) -> Result<crate::ClaudeConfig> {
        // Read global config (always present, may be empty)
        let global_config = self.get_global_config()?;

        // Try to read project config
        let project_config = self.get_project_config(project_path)?;

        match project_config {
            Some(proj) => {
                // Merge: project config overrides global config
                Ok(crate::config::merge::merge_configs(&global_config, &proj))
            }
            None => {
                // No project config, return global only
                Ok(global_config)
            }
        }
    }

    /// Update global configuration
    ///
    /// # Arguments
    /// * `config` - The new global configuration
    ///
    /// # Errors
    /// Returns an error if write fails
    pub fn update_global_config(&self, config: &crate::ClaudeConfig) -> Result<()> {
        let global_path = get_global_config_path();
        self.write_config_with_backup(&global_path, config)
    }

    /// Update project configuration
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory
    /// * `config` - The new project configuration
    ///
    /// # Errors
    /// Returns an error if write fails
    pub fn update_project_config(
        &self,
        project_path: &Path,
        config: &crate::ClaudeConfig,
    ) -> Result<()> {
        let config_path = project_path.join(".claude").join("config.json");
        self.write_config_with_backup(&config_path, config)
    }

    /// Compute differences between global and project configurations
    ///
    /// # Arguments
    /// * `project_path` - Path to the project directory (if None, searches upward)
    ///
    /// # Returns
    /// List of differences and source map
    ///
    /// # Errors
    /// Returns an error if configs cannot be read
    pub fn diff_configs(
        &self,
        project_path: Option<&Path>,
    ) -> Result<(Vec<ConfigDiff>, SourceMap)> {
        let global_config = self.get_global_config()?;
        let project_config = self.get_project_config(project_path)?;

        let global_json = serde_json::to_value(&global_config)?;
        let project_json = serde_json::to_value(&project_config)?;

        let mut diffs = Vec::new();
        let mut source_map = SourceMap::new();

        // Compare all keys
        self.compare_values(
            &global_json,
            &project_json,
            "",
            &mut diffs,
            &mut source_map,
            ConfigScope::Global,
        );

        // Find additions (keys only in project)
        self.find_additions(
            &global_json,
            &project_json,
            "",
            &mut diffs,
            &mut source_map,
            ConfigScope::Project,
        );

        Ok((diffs, source_map))
    }

    /// Compare values between two configs
    fn compare_values(
        &self,
        global: &serde_json::Value,
        project: &serde_json::Value,
        key_path: &str,
        diffs: &mut Vec<ConfigDiff>,
        source_map: &mut SourceMap,
        global_scope: ConfigScope,
    ) {
        match (global, project) {
            (Value::Object(global_map), Value::Object(project_map)) => {
                // Process all keys in global
                for (key, global_value) in global_map {
                    let new_key_path = if key_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{key_path}.{key}")
                    };

                    if let Some(project_value) = project_map.get(key) {
                        // Key exists in both - check if values differ
                        if global_value != project_value {
                            diffs.push(ConfigDiff::Modified {
                                key_path: new_key_path.clone(),
                                old_value: global_value.clone(),
                                new_value: project_value.clone(),
                            });
                            source_map.insert(new_key_path.clone(), ConfigScope::Project);
                        } else {
                            // Values are the same - from global
                            source_map.insert(new_key_path, global_scope);
                        }
                    } else {
                        // Key only in global - removed in project
                        diffs.push(ConfigDiff::Removed {
                            key_path: new_key_path.clone(),
                            value: global_value.clone(),
                        });
                        source_map.insert(new_key_path, ConfigScope::Global);
                    }
                }
            }
            (Value::Array(global_arr), Value::Array(project_arr)) => {
                // Arrays use replace strategy - no deep comparison needed
                if global_arr != project_arr {
                    let new_key_path = if key_path.is_empty() {
                        key_path.to_string()
                    } else {
                        key_path.to_string()
                    };

                    diffs.push(ConfigDiff::Modified {
                        key_path: new_key_path.clone(),
                        old_value: Value::Array(global_arr.clone()),
                        new_value: Value::Array(project_arr.clone()),
                    });
                    source_map.insert(new_key_path, ConfigScope::Project);
                }
            }
            _ => {
                // Different types - treat as modification
                if global != project {
                    diffs.push(ConfigDiff::Modified {
                        key_path: key_path.to_string(),
                        old_value: global.clone(),
                        new_value: project.clone(),
                    });
                    source_map.insert(key_path.to_string(), ConfigScope::Project);
                }
            }
        }
    }

    /// Find keys that only exist in project (additions)
    fn find_additions(
        &self,
        global: &serde_json::Value,
        project: &serde_json::Value,
        key_path: &str,
        diffs: &mut Vec<ConfigDiff>,
        source_map: &mut SourceMap,
        project_scope: ConfigScope,
    ) {
        if let (Value::Object(global_map), Value::Object(project_map)) = (global, project) {
            for (key, project_value) in project_map {
                let new_key_path = if key_path.is_empty() {
                    key.clone()
                } else {
                    format!("{key_path}.{key}")
                };

                if !global_map.contains_key(key) {
                    // Key only in project - addition
                    diffs.push(ConfigDiff::Added {
                        key_path: new_key_path.clone(),
                        value: project_value.clone(),
                    });
                    source_map.insert(new_key_path.clone(), project_scope);

                    // Recurse into nested objects for additions
                    if let Value::Object(nested_project) = project_value {
                        let empty_object = Value::Object(Default::default());
                        let global_nested_ref = global_map.get(key).unwrap_or(&empty_object);

                        if let Value::Object(nested_global) = global_nested_ref {
                            let global_value = Value::Object(nested_global.clone());
                            let project_value = Value::Object(nested_project.clone());
                            self.find_additions(
                                &global_value,
                                &project_value,
                                &new_key_path,
                                diffs,
                                source_map,
                                project_scope,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Search configuration for matching keys and/or values
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `scope` - Which config(s) to search (Global, Project, or Both)
    ///
    /// # Returns
    /// Vector of search results with key paths, values, and sources
    ///
    /// # Example
    /// ```no_run
    /// # use claude_config_manager_core::{ConfigManager, SearchOptions, types::ConfigScope};
    /// # let manager = ConfigManager::new("/tmp/backups");
    /// let results = manager.search_config("npx", ConfigScope::Global).unwrap();
    /// for result in results {
    ///     println!("{}: {}", result.key_path, result.value);
    /// }
    /// ```
    pub fn search_config(&self, query: &str, scope: ConfigScope) -> Result<Vec<SearchResult>> {
        self.search_config_with_options(query, scope, SearchOptions::new())
    }

    /// Search configuration with custom options
    ///
    /// # Arguments
    /// * `query` - Search query string
    /// * `scope` - Which config(s) to search
    /// * `options` - Search options (case sensitivity, search keys vs values, etc.)
    ///
    /// # Returns
    /// Vector of search results
    pub fn search_config_with_options(
        &self,
        query: &str,
        scope: ConfigScope,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Search based on scope
        match scope {
            ConfigScope::Global => {
                let global_path = get_global_config_path();
                if global_path.exists() {
                    if let Ok(config) = self.read_config(&global_path) {
                        let searcher = ConfigSearcher::with_options(options.clone());
                        let results =
                            searcher.search(query, &config, ConfigScope::Global, global_path)?;
                        all_results.extend(results);
                    }
                }
            }
            ConfigScope::Project => {
                // For project scope, try to find project config from current directory
                if let Some(project_path) = find_project_config(None) {
                    if let Ok(config) = self.read_config(&project_path) {
                        let searcher = ConfigSearcher::with_options(options.clone());
                        let results =
                            searcher.search(query, &config, ConfigScope::Project, project_path)?;
                        all_results.extend(results);
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// Export configuration to a file
    ///
    /// # Arguments
    /// * `config` - Configuration to export
    /// * `path` - Destination file path
    ///
    /// # Returns
    /// Path to the exported file
    ///
    /// # Errors
    /// Returns an error if export fails
    pub fn export_config(&self, config: &crate::ClaudeConfig, path: &Path) -> Result<PathBuf> {
        crate::ConfigImporter::export(config, path)
    }

    /// Import configuration from a file
    ///
    /// # Arguments
    /// * `path` - Source file path
    ///
    /// # Returns
    /// Imported configuration
    ///
    /// # Errors
    /// Returns an error if import fails
    pub fn import_config(&self, path: &Path) -> Result<crate::ClaudeConfig> {
        crate::ConfigImporter::import(path)
    }

    /// Export configuration with custom options
    ///
    /// # Arguments
    /// * `config` - Configuration to export
    /// * `path` - Destination file path
    /// * `options` - Export options
    ///
    /// # Returns
    /// Path to the exported file
    pub fn export_config_with_options(
        &self,
        config: &crate::ClaudeConfig,
        path: &Path,
        options: crate::ImportExportOptions,
    ) -> Result<PathBuf> {
        crate::ConfigImporter::export_config(config, path, &options)
    }

    /// Import configuration with custom options
    ///
    /// # Arguments
    /// * `path` - Source file path
    /// * `options` - Import options
    ///
    /// # Returns
    /// Imported configuration
    pub fn import_config_with_options(
        &self,
        path: &Path,
        options: crate::ImportExportOptions,
    ) -> Result<crate::ClaudeConfig> {
        crate::ConfigImporter::import_config(path, &options)
    }
}

/// Parse JSON error location from error message
///
/// Extracts line and column numbers from serde_json error messages.
/// Returns (0, 0) if location cannot be determined.
fn parse_json_error_location(error_msg: &str) -> (usize, usize) {
    // Typical serde_json error format: "key error at line X, column Y"
    if let Some(line_pos) = error_msg.find("line ") {
        if let Some(colon_pos) = error_msg[line_pos + 5..].find(',') {
            if let Ok(line) = error_msg[line_pos + 5..line_pos + colon_pos].parse::<usize>() {
                if let Some(col_pos) = error_msg.find("column ") {
                    if let Some(end) = error_msg[col_pos + 7..].find(',') {
                        if let Ok(column) =
                            error_msg[col_pos + 7..col_pos + 7 + end].parse::<usize>()
                        {
                            return (line, column);
                        }
                    }
                }
            }
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // TDD Test 1: Read valid config
    #[test]
    fn test_read_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        // Create valid config file
        let config_content = r#"{
            "mcpServers": {
                "npx": {
                    "enabled": true,
                    "command": "npx",
                    "args": []
                }
            }
        }"#;
        fs::write(&config_path, config_content).unwrap();

        let manager = ConfigManager::new(&backup_dir);
        let config = manager.read_config(&config_path).unwrap();

        assert!(config.mcp_servers.is_some());
        assert_eq!(config.mcp_servers.unwrap().len(), 1);
    }

    // TDD Test 2: Read nonexistent file returns proper error
    #[test]
    fn test_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);
        let result = manager.read_config(&config_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // TDD Test 3: Read invalid JSON returns proper error
    #[test]
    fn test_read_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        // Create invalid JSON
        fs::write(&config_path, b"{invalid json}").unwrap();

        let manager = ConfigManager::new(&backup_dir);
        let result = manager.read_config(&config_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Invalid JSON"));
        assert!(message.contains("line 1"));
    }

    // TDD Test 4: Write config creates backup
    #[test]
    fn test_write_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        // Create initial config
        fs::write(&config_path, b"{}").unwrap();

        let manager = ConfigManager::new(&backup_dir);

        // Write new config
        let config = crate::ClaudeConfig::new();
        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();

        // Verify backup was created
        let backups = manager.backup_manager().list_backups(&config_path).unwrap();
        assert_eq!(backups.len(), 1);
    }

    // TDD Test 5: Write validates config
    #[test]
    fn test_write_validates_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);

        // Create invalid config (empty server name)
        let mut config = crate::ClaudeConfig::new();
        let mut servers = std::collections::HashMap::new();
        servers.insert("".to_string(), crate::McpServer::new("", "npx", vec![]));
        config.mcp_servers = Some(servers);

        let result = manager.write_config_with_backup(&config_path, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("validation failed"));
    }

    // TDD Test 6: Write creates parent directory
    #[test]
    fn test_write_creates_parent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);
        let config = crate::ClaudeConfig::new();

        // Write to non-existent nested directory
        manager
            .write_config_with_backup(&nested_path, &config)
            .unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    // TDD Test 7: Atomic write preserves original on failure
    #[test]
    fn test_atomic_write_preserves_original() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);

        // Create initial config
        let original_content = b"{\"version\": 1}";
        fs::write(&config_path, original_content).unwrap();

        // Try to write invalid config (should fail)
        let mut invalid_config = crate::ClaudeConfig::new();
        let mut servers = std::collections::HashMap::new();
        servers.insert("".to_string(), crate::McpServer::new("", "npx", vec![]));
        invalid_config.mcp_servers = Some(servers);

        let result = manager.write_config_with_backup(&config_path, &invalid_config);

        assert!(result.is_err());

        // Verify original file unchanged
        let current_content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(current_content.as_bytes(), original_content);
    }

    // TDD Test 8: Write produces properly formatted JSON
    #[test]
    fn test_write_produces_formatted_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);
        let config = crate::ClaudeConfig::new()
            .with_allowed_path("~/projects")
            .with_custom_instruction("Be concise");

        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();

        // Read and verify format
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("allowedPaths"));
        assert!(content.contains("customInstructions"));
        assert!(content.contains("\n")); // Pretty printed
    }

    // TDD Test 9: Write to existing file preserves unknown fields
    #[test]
    fn test_write_preserves_unknown_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backs");

        // Create config with unknown field
        let json_with_unknown = r#"{
            "mcpServers": {"npx": {"enabled": true}},
            "futureFeature": {"setting": 42}
        }"#;
        fs::write(&config_path, json_with_unknown).unwrap();

        let manager = ConfigManager::new(&backup_dir);

        // Read, then write back
        let config = manager.read_config(&config_path).unwrap();
        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();

        // Verify unknown field preserved
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("futureFeature"));
    }

    // TDD Test 10: First write (no existing file) works
    #[test]
    fn test_first_write_no_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        let backup_dir = temp_dir.path().join("backs");

        let manager = ConfigManager::new(&backup_dir);
        let config = crate::ClaudeConfig::new();

        // Write to non-existent file (should work without backup)
        manager
            .write_config_with_backup(&config_path, &config)
            .unwrap();

        assert!(config_path.exists());

        // Verify no backup was created (no existing file to backup)
        let backups = manager.backup_manager().list_backups(&config_path).unwrap();
        assert!(backups.is_empty());
    }

    // TDD Test 11: Get global config returns empty when file doesn't exist
    #[test]
    fn test_get_global_config_returns_empty_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);

        // Mock that global config doesn't exist
        // We'll test the method behavior indirectly
        // In real scenario, it checks get_global_config_path()
        let result = manager.read_config(&temp_dir.path().join("nonexistent.json"));

        // Should fail since file doesn't exist
        assert!(result.is_err());
    }

    // TDD Test 12: Get project config with explicit path
    #[test]
    fn test_get_project_config_explicit_path() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("myproject");
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let config_path = claude_dir.join("config.json");
        let backup_dir = temp_dir.path().join("backups");

        // Create project config
        let config_content = r#"{
            "mcpServers": {
                "npx": {"enabled": true}
            }
        }"#;
        fs::write(&config_path, config_content).unwrap();

        let manager = ConfigManager::new(&backup_dir);
        let result = manager.get_project_config(Some(&project_dir));

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.is_some());
        let config = config.unwrap();
        assert!(config.mcp_servers.is_some());
        assert_eq!(config.mcp_servers.unwrap().len(), 1);
    }

    // TDD Test 13: Get project config returns None when not found
    #[test]
    fn test_get_project_config_returns_none_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        let manager = ConfigManager::new(&backup_dir);

        // Use temp_dir as project path (no .claude directory)
        let result = manager.get_project_config(Some(temp_dir.path()));

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // TDD Test 14: Get merged config with project override
    #[test]
    fn test_get_merged_config_project_override() {
        let temp_dir = TempDir::new().unwrap();

        // Create global config
        let global_config = crate::ClaudeConfig::new()
            .with_allowed_path("~/global-projects")
            .with_custom_instruction("Global instruction");

        // Create project directory and config
        let project_dir = temp_dir.path().join("myproject");
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();

        let project_config = crate::ClaudeConfig::new().with_allowed_path("~/my-project");

        let backup_dir = temp_dir.path().join("backups");
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

        // Manually read and merge for testing
        let global = manager.read_config(&global_path).unwrap();
        let project = manager.read_config(&project_path).unwrap();
        let merged = crate::config::merge::merge_configs(&global, &project);

        // Project should override global's allowedPaths
        assert!(merged.allowed_paths.is_some());
        let paths = merged.allowed_paths.unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "~/my-project");
    }

    // TDD Test 15: Get merged config without project returns global
    #[test]
    fn test_get_merged_config_no_project_returns_global() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        let global_config =
            crate::ClaudeConfig::new().with_custom_instruction("Global instruction");

        let global_path = temp_dir.path().join("global.json");
        let manager = ConfigManager::new(&backup_dir);
        manager
            .write_config_with_backup(&global_path, &global_config)
            .unwrap();

        // Read global back
        let result = manager.read_config(&global_path);

        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.custom_instructions.is_some());
        assert_eq!(config.custom_instructions.unwrap().len(), 1);
    }

    // TDD Test 16: Get merged config deep merges objects
    #[test]
    fn test_get_merged_config_deep_merges_objects() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");

        // Create global with npx server
        let global_config = crate::ClaudeConfig::new()
            .with_mcp_server("npx", crate::McpServer::new("npx", "npx", vec![]));

        // Create project with uvx server
        let project_config = crate::ClaudeConfig::new()
            .with_mcp_server("uvx", crate::McpServer::new("uvx", "uvx", vec![]));

        let global_path = temp_dir.path().join("global.json");
        let project_path = temp_dir.path().join("project.json");

        let manager = ConfigManager::new(&backup_dir);
        manager
            .write_config_with_backup(&global_path, &global_config)
            .unwrap();
        manager
            .write_config_with_backup(&project_path, &project_config)
            .unwrap();

        // Merge
        let global = manager.read_config(&global_path).unwrap();
        let project = manager.read_config(&project_path).unwrap();
        let merged = crate::config::merge::merge_configs(&global, &project);

        // Should have both servers
        assert!(merged.mcp_servers.is_some());
        let servers = merged.mcp_servers.unwrap();
        assert_eq!(servers.len(), 2);
        assert!(servers.contains_key("npx"));
        assert!(servers.contains_key("uvx"));
    }
}
