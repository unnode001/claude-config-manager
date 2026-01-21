//! Tauri commands for MCP server management

use crate::commands::types::*;
use crate::commands::config::ConfigState;
use claude_config_manager_core::{ConfigScope, McpManager, McpServer};
use std::path::PathBuf;
use tauri::State;

/// List all MCP servers
#[tauri::command]
pub async fn list_servers(
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<Vec<McpServerData>, String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let project_path_buf = project_path.map(PathBuf::from);
    let servers = manager
        .list_servers(&config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(servers
        .into_iter()
        .map(|(name, mut server)| {
            server.name = name;
            McpServerData::from(server)
        })
        .collect())
}

/// Add a new MCP server
#[tauri::command]
pub async fn add_server(
    name: String,
    command: String,
    args: Option<Vec<String>>,
    env: Option<Vec<String>>,
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<(), String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let mut server = McpServer::new(&name, &command, args.unwrap_or_default());

    if let Some(env_vars) = env {
        for env_var in env_vars {
            if let Some((key, value)) = env_var.split_once('=') {
                server.env.insert(key.to_string(), value.to_string());
            }
        }
    }

    let project_path_buf = project_path.map(PathBuf::from);
    manager
        .add_server(&name, server, &config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove an MCP server
#[tauri::command]
pub async fn remove_server(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<(), String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let project_path_buf = project_path.map(PathBuf::from);
    manager
        .remove_server(&name, &config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Enable an MCP server
#[tauri::command]
pub async fn enable_server(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<(), String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let project_path_buf = project_path.map(PathBuf::from);
    manager
        .enable_server(&name, &config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Disable an MCP server
#[tauri::command]
pub async fn disable_server(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<(), String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let project_path_buf = project_path.map(PathBuf::from);
    manager
        .disable_server(&name, &config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Get details of a specific server
#[tauri::command]
pub async fn get_server(
    name: String,
    scope: Option<String>,
    project_path: Option<String>,
    _state: State<'_, ConfigState>,
) -> Result<McpServerData, String> {
    let backup_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("claude")
        .join("backups");

    let manager = McpManager::new(&backup_dir);
    let config_scope = parse_scope(&scope, &project_path)?;

    let project_path_buf = project_path.map(PathBuf::from);
    let mut server = manager
        .get_server(&name, &config_scope, project_path_buf.as_deref())
        .map_err(|e| e.to_string())?;

    server.name = name.clone();
    Ok(McpServerData::from(server))
}

fn parse_scope(scope: &Option<String>, project_path: &Option<String>) -> Result<ConfigScope, String> {
    match (scope.as_deref(), project_path) {
        (Some("project"), _) => Ok(ConfigScope::Project),
        (Some("global"), _) => Ok(ConfigScope::Global),
        (None, Some(_)) => Ok(ConfigScope::Project),
        (None, None) => Ok(ConfigScope::Global),
        _ => Err("Invalid scope".to_string()),
    }
}
