//! Configuration import/export functionality
//!
//! Provides functionality to export configurations to files
//! and import configurations from files with validation.

use crate::{config::ClaudeConfig, error::ConfigError, error::Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Supported export formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// TOML format (future support)
    Toml,
}

impl ExportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &str {
        match self {
            ExportFormat::Json => "json",
            ExportFormat::Toml => "toml",
        }
    }

    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| match ext {
                "json" => Some(ExportFormat::Json),
                "toml" => Some(ExportFormat::Toml),
                _ => None,
            })
    }
}

/// Import/export options
#[derive(Debug, Clone)]
pub struct ImportExportOptions {
    /// Export format
    pub format: ExportFormat,

    /// Whether to validate before import
    pub validate: bool,

    /// Whether to create backup before import
    pub backup: bool,

    /// Pretty print JSON output
    pub pretty: bool,
}

impl Default for ImportExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            validate: true,
            backup: true,
            pretty: true,
        }
    }
}

/// Configuration importer/exporter
pub struct ConfigImporter;

impl ConfigImporter {
    /// Export configuration to a file
    ///
    /// # Arguments
    /// * `config` - Configuration to export
    /// * `path` - Destination file path
    /// * `options` - Export options
    ///
    /// # Errors
    /// Returns an error if:
    /// - File cannot be created
    /// - Serialization fails
    pub fn export_config(
        config: &ClaudeConfig,
        path: &Path,
        options: &ImportExportOptions,
    ) -> Result<PathBuf> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ConfigError::filesystem("create export directory", parent, e))?;
            }
        }

        // Serialize based on format
        let content = match options.format {
            ExportFormat::Json => {
                if options.pretty {
                    serde_json::to_string_pretty(config)
                } else {
                    serde_json::to_string(config)
                }
            }
            ExportFormat::Toml => {
                // TOML support can be added later with the toml crate
                return Err(ConfigError::validation_failed(
                    "ExportFormat",
                    "TOML format is not yet supported",
                    "Use JSON format instead",
                ));
            }
        }
        .map_err(|e| ConfigError::Generic(format!("Serialization failed: {e}")))?;

        // Write to file
        let mut file = fs::File::create(path)
            .map_err(|e| ConfigError::filesystem("create export file", path, e))?;

        file.write_all(content.as_bytes())
            .map_err(|e| ConfigError::filesystem("write export file", path, e))?;

        tracing::info!("Exported configuration to: {}", path.display());

        Ok(path.to_path_buf())
    }

    /// Import configuration from a file
    ///
    /// # Arguments
    /// * `path` - Source file path
    /// * `options` - Import options
    ///
    /// # Returns
    /// Imported configuration
    ///
    /// # Errors
    /// Returns an error if:
    /// - File doesn't exist
    /// - File cannot be read
    /// - Deserialization fails
    /// - Validation fails (if enabled)
    pub fn import_config(path: &Path, options: &ImportExportOptions) -> Result<ClaudeConfig> {
        // Check file exists
        if !path.exists() {
            return Err(ConfigError::not_found(path));
        }

        // Read file content
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::filesystem("read import file", path, e))?;

        // Detect format from path if not specified
        let format = ExportFormat::from_path(path).unwrap_or(options.format);

        // Deserialize based on format
        let config = match format {
            ExportFormat::Json => serde_json::from_str(&content)
                .map_err(|e| ConfigError::Generic(format!("Failed to parse JSON: {e}")))?,
            ExportFormat::Toml => {
                return Err(ConfigError::validation_failed(
                    "ImportFormat",
                    "TOML format is not yet supported",
                    "Use JSON format instead",
                ));
            }
        };

        // Validate if requested
        if options.validate {
            crate::validate_config(&config)?;
        }

        tracing::info!("Imported configuration from: {}", path.display());

        Ok(config)
    }

    /// Export configuration with default options
    ///
    /// Convenience method for common export operations
    pub fn export(config: &ClaudeConfig, path: &Path) -> Result<PathBuf> {
        Self::export_config(config, path, &ImportExportOptions::default())
    }

    /// Import configuration with default options
    ///
    /// Convenience method for common import operations
    pub fn import(path: &Path) -> Result<ClaudeConfig> {
        Self::import_config(path, &ImportExportOptions::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::McpServer;
    use tempfile::TempDir;

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Toml.extension(), "toml");
    }

    #[test]
    fn test_export_format_from_path() {
        let json_path = PathBuf::from("/test/config.json");
        let toml_path = PathBuf::from("/test/config.toml");
        let txt_path = PathBuf::from("/test/config.txt");

        assert_eq!(
            ExportFormat::from_path(&json_path),
            Some(ExportFormat::Json)
        );
        assert_eq!(
            ExportFormat::from_path(&toml_path),
            Some(ExportFormat::Toml)
        );
        assert_eq!(ExportFormat::from_path(&txt_path), None);
    }

    #[test]
    fn test_export_import_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("export.json");

        // Create a test config
        let original_config = ClaudeConfig::new()
            .with_mcp_server("test", McpServer::new("cmd", "cmd", vec!["-y".to_string()]))
            .with_custom_instruction("Test instruction");

        // Export
        let result = ConfigImporter::export(&original_config, &export_path);
        assert!(result.is_ok());
        assert!(export_path.exists());

        // Import
        let imported_config = ConfigImporter::import(&export_path).unwrap();

        // Verify
        assert!(imported_config.mcp_servers.is_some());
        let servers = imported_config.mcp_servers.unwrap();
        assert!(servers.contains_key("test"));
        assert!(imported_config.custom_instructions.is_some());
        let instructions = imported_config.custom_instructions.unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], "Test instruction");
    }

    #[test]
    fn test_export_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("dir")
            .join("config.json");

        let config = ClaudeConfig::new();
        let result = ConfigImporter::export(&config, &nested_path);

        assert!(result.is_ok());
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_import_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.json");

        let result = ConfigImporter::import(&nonexistent);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&invalid_path, "{ invalid json }").unwrap();

        let result = ConfigImporter::import(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_options_default() {
        let options = ImportExportOptions::default();

        assert_eq!(options.format, ExportFormat::Json);
        assert!(options.validate);
        assert!(options.backup);
        assert!(options.pretty);
    }
}
