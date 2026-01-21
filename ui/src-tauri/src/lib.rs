//! Claude Config Manager - GUI Application
//!
//! Tauri-based desktop application for managing Claude Code configurations.

use crate::commands::config::ConfigState;
use crate::commands::*;

mod commands;

// Re-exports
pub use tauri;

/// Tauri application entry point
pub fn run() {
    let config_state = ConfigState::new();

    tauri::Builder::default()
        .manage(config_state)
        .invoke_handler(tauri::generate_handler![
            // Configuration commands
            commands::config::get_config,
            commands::config::set_config_value,

            // Project commands
            commands::project::list_projects,
            commands::project::get_project_config,

            // MCP server commands
            commands::mcp::list_servers,
            commands::mcp::add_server,
            commands::mcp::remove_server,
            commands::mcp::enable_server,
            commands::mcp::disable_server,
            commands::mcp::get_server,

            // History commands
            commands::history::list_backups,
            commands::history::restore_backup,

            // Utility commands
            commands::utils::get_global_config_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
