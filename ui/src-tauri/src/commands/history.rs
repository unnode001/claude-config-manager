//! Tauri commands for history management

use crate::commands::config::ConfigState;
use crate::commands::types::*;
use claude_config_manager_core::BackupInfo;
use std::path::PathBuf;
use tauri::State;

/// List all backups
#[tauri::command]
pub async fn list_backups(
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<Vec<BackupInfoData>, String> {
    let manager = &state.manager.backup_manager();

    let config_file = if let Some(project) = project_path {
        PathBuf::from(project).join(".claude").join("config.json")
    } else {
        claude_config_manager_core::get_global_config_path()
    };

    let backups = manager
        .list_backups(&config_file)
        .map_err(|e| e.to_string())?;

    Ok(backups
        .into_iter()
        .map(BackupInfoData::from)
        .collect())
}

/// Restore from a backup
#[tauri::command]
pub async fn restore_backup(
    backup_path: String,
    state: State<'_, ConfigState>,
) -> Result<(), String> {
    let manager = &state.manager.backup_manager();

    manager
        .restore_backup(&PathBuf::from(backup_path))
        .map_err(|e| e.to_string())?;

    Ok(())
}
