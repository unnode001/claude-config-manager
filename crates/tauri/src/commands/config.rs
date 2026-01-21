//! Tauri commands for configuration management

use crate::types::*;
use claude_config_manager_core::{ConfigManager, MergeResult};
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;

pub mod types;

/// Application state for ConfigManager
pub struct ConfigState {
    pub manager: ConfigManager,
}

impl ConfigState {
    pub fn new() -> Self {
        // Get default backup directory
        let backup_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("claude")
            .join("backups");

        Self {
            manager: ConfigManager::new(&backup_dir),
        }
    }
}

/// Get current configuration
#[tauri::command]
pub async fn get_config(
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<ClaudeConfigData, String> {
    let manager = &state.manager;

    let config = if let Some(path) = project_path {
        manager
            .get_merged_config(Some(&PathBuf::from(path)))
            .map_err(|e| e.to_string())?
    } else {
        manager.get_global_config().map_err(|e| e.to_string())?
    };

    Ok(ClaudeConfigData::from(config))
}

/// Set a configuration value by key path
#[tauri::command]
pub async fn set_config_value(
    key: String,
    value: Value,
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<(), String> {
    let manager = &state.manager;

    // Determine config file path
    let config_path = if let Some(project) = project_path {
        PathBuf::from(project).join(".claude").join("config.json")
    } else {
        claude_config_manager_core::get_global_config_path()
    };

    // Read current config
    let mut config = if config_path.exists() {
        manager.read_config(&config_path).map_err(|e| e.to_string())?
    } else {
        claude_config_manager_core::ClaudeConfig::new()
    };

    // Parse key path and set value
    let keys: Vec<&str> = key.split('.').collect();
    crate::commands::set_value_by_key_path(&mut config, &keys, value)
        .map_err(|e| e.to_string())?;

    // Write with backup
    manager
        .write_config_with_backup(&config_path, &config)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Compare global and project configurations
#[tauri::command]
pub async fn diff_configs(
    project_path: String,
    state: State<'_, ConfigState>,
) -> Result<ConfigDiffData, String> {
    let manager = &state.manager;

    let diff = manager
        .diff_configs(Some(&PathBuf::from(project_path)))
        .map_err(|e| e.to_string())?;

    Ok(ConfigDiffData::from(diff))
}

/// Import configuration from file
#[tauri::command]
pub async fn import_config(
    file_path: String,
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<(), String> {
    use claude_config_manager_core::ConfigImporter;

    let importer = ConfigImporter::new(&state.manager);
    let options = claude_config_manager_core::ImportExportOptions {
        validate: true,
        merge: true,
    };

    let config_path = if let Some(project) = project_path {
        PathBuf::from(project).join(".claude").join("config.json")
    } else {
        claude_config_manager_core::get_global_config_path()
    };

    importer
        .import_config(&PathBuf::from(file_path), &config_path, &options)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Export configuration to file
#[tauri::command]
pub async fn export_config(
    file_path: String,
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<(), String> {
    use claude_config_manager_core::{ConfigExporter, ExportFormat};

    let exporter = ConfigExporter::new(&state.manager);

    let config_path = if let Some(project) = project_path {
        PathBuf::from(project).join(".claude").join("config.json")
    } else {
        claude_config_manager_core::get_global_config_path()
    };

    // Detect format from file extension
    let format = if file_path.ends_with(".toml") {
        ExportFormat::Toml
    } else {
        ExportFormat::Json
    };

    exporter
        .export_config(&config_path, &PathBuf::from(file_path), format)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Helper function to set value by key path
fn set_value_by_key_path(
    config: &mut claude_config_manager_core::ClaudeConfig,
    keys: &[&str],
    value: Value,
) -> Result<(), String> {
    // This is a simplified version - the full implementation would be similar to
    // the CLI's key_path.rs module
    if keys.len() == 1 {
        match keys[0] {
            "customInstructions" => {
                if let Some(s) = value.as_str() {
                    config.custom_instructions = Some(vec![s.to_string()]);
                }
            }
            _ => {
                // Add to unknown fields
                config
                    .unknown
                    .insert(keys[0].to_string(), value);
            }
        }
    }
    Ok(())
}
