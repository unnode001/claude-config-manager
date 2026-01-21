//! Utility commands

use claude_config_manager_core;

/// Get the global configuration file path
#[tauri::command]
pub async fn get_global_config_path() -> Result<String, String> {
    Ok(claude_config_manager_core::get_global_config_path()
        .to_string_lossy()
        .to_string())
}

/// Get the application version
#[tauri::command]
pub async fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
