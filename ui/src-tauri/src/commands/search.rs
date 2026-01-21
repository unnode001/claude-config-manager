//! Tauri commands for search functionality

use crate::commands::config::ConfigState;
use crate::commands::types::*;
use claude_config_manager_core::{ConfigSearcher, ConfigScope, SearchOptions, ValueType};
use std::path::PathBuf;
use tauri::State;

/// Search configuration values
#[tauri::command]
pub async fn search_config(
    query: String,
    search_keys: Option<bool>,
    search_values: Option<bool>,
    case_sensitive: Option<bool>,
    regex: Option<bool>,
    project_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<Vec<SearchResultData>, String> {
    let manager = &state.manager;

    let options = SearchOptions {
        search_keys: search_keys.unwrap_or(true),
        search_values: search_values.unwrap_or(false),
        case_sensitive: case_sensitive.unwrap_or(false),
        regex: regex.unwrap_or(false),
        max_depth: None,
    };

    let searcher = ConfigSearcher::with_options(options);

    let (config, scope, config_path) = if let Some(path) = project_path {
        let path_buf = PathBuf::from(&path);
        let config_path = path_buf.join(".claude").join("config.json");
        let config = manager
            .get_project_config(Some(&path_buf))
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        (config, ConfigScope::Project, config_path)
    } else {
        let config_path = claude_config_manager_core::get_global_config_path();
        let config = manager.get_global_config().map_err(|e| e.to_string())?;
        (config, ConfigScope::Global, config_path)
    };

    let results = searcher
        .search(&query, &config, scope, config_path)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(SearchResultData::from)
        .collect())
}
