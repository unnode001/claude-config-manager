//! Tauri commands for project management

use crate::commands::config::ConfigState;
use crate::commands::types::*;
use claude_config_manager_core::ProjectScanner;
use std::path::PathBuf;
use tauri::State;

/// Scan directory for projects
#[tauri::command]
pub async fn scan_projects(
    path: String,
    max_depth: Option<usize>,
    state: State<'_, ConfigState>,
) -> Result<Vec<ProjectData>, String> {
    let scanner = ProjectScanner::new(max_depth, false);

    let projects = scanner
        .scan_directory(&PathBuf::from(path))
        .map_err(|e| e.to_string())?;

    Ok(projects.into_iter().map(ProjectData::from).collect())
}

/// List all discovered projects
#[tauri::command]
pub async fn list_projects(
    state: State<'_, ConfigState>,
) -> Result<Vec<ProjectData>, String> {
    // Get user's home directory
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    let scanner = ProjectScanner::new(Some(3), false); // Scan up to 3 levels deep
    let projects = scanner
        .scan_directory(&home)
        .map_err(|e| e.to_string())?;

    Ok(projects.into_iter().map(ProjectData::from).collect())
}

/// Get project configuration
#[tauri::command]
pub async fn get_project_config(
    project_path: String,
    state: State<'_, ConfigState>,
) -> Result<ClaudeConfigData, String> {
    let manager = &state.manager;

    let config = manager
        .get_project_config(&PathBuf::from(project_path))
        .map_err(|e| e.to_string())?;

    Ok(ClaudeConfigData::from(config))
}
