//! Claude Config Manager - Core Library
//!
//! This library provides the core functionality for managing Claude Code configuration files.
//! It is designed to be frontend-agnostic and can be used by CLI, GUI, or other interfaces.

// Public modules
pub mod backup;
pub mod config;
pub mod error;
pub mod mcp;
pub mod paths;
pub mod project;
pub mod types;

// Validation is part of config module
pub use config::validation::validate_config;

// Private modules (will be added as we implement features)
// mod skills;
// mod project;

// Re-exports for convenience
pub use backup::BackupManager;
pub use config::{ClaudeConfig, manager::ConfigManager, merge::merge_configs};
pub use error::{ConfigError, Result};
pub use mcp::McpManager;
pub use paths::{expand_tilde, find_project_config, get_global_config_dir, get_global_config_path};
pub use project::{ProjectInfo, ProjectScanner};
pub use types::*;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");
