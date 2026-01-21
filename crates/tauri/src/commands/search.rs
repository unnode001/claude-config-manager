//! Tauri commands for search functionality

use crate::commands::config::ConfigState;
use crate::commands::types::*;
use claude_config_manager_core::{ConfigSearcher, SearchOptions, ValueType};
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

    let searcher = ConfigSearcher::new(options);

    let config = if let Some(path) = project_path {
        manager
            .get_merged_config(Some(&std::path::PathBuf::from(path)))
            .map_err(|e| e.to_string())?
    } else {
        manager.get_global_config().map_err(|e| e.to_string())?
    };

    let results = searcher
        .search(&query, &config)
        .map_err(|e| e.to_string())?;

    Ok(results
        .into_iter()
        .map(SearchResultData::from)
        .collect())
}
