//! Tauri commands for configuration management

use crate::types::*;
use claude_config_manager_core::ConfigManager;
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;

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
            .get_project_config(Some(PathBuf::from(path).as_path()))
            .map_err(|e| e.to_string())?
            .unwrap_or_default()
    } else {
        manager.get_global_config().map_err(|e| e.to_string())?
    };

    Ok(ClaudeConfigData::from(config))
}

/// Set a configuration value by key path (simplified)
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

    // Parse key path and set value (simplified)
    let keys: Vec<&str> = key.split('.').collect();
    set_value_by_key_path(&mut config, &keys, value)
        .map_err(|e| e.to_string())?;

    // Write with backup
    manager
        .write_config_with_backup(&config_path, &config)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Helper function to set value by key path
fn set_value_by_key_path(
    config: &mut claude_config_manager_core::ClaudeConfig,
    keys: &[&str],
    value: Value,
) -> Result<(), String> {
    // This is a simplified version - only handles basic cases
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
