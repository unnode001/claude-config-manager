//! Search command
//!
//! Implements `search` command for finding configuration values

use anyhow::Result;
use clap::Parser;
use claude_config_manager_core::{
    ConfigManager, SearchOptions,
    types::ConfigScope,
};

/// Search command arguments
#[derive(Parser, Debug)]
pub struct SearchArgs {
    /// Search query
    query: String,

    /// Search in values instead of keys
    #[arg(short, long)]
    value: bool,

    /// Search in both keys and values
    #[arg(short = 'b', long)]
    both: bool,

    /// Case sensitive search
    #[arg(short = 'c', long)]
    case_sensitive: bool,

    /// Maximum search depth
    #[arg(short = 'd', long)]
    depth: Option<usize>,

    /// Search in global config
    #[arg(long)]
    global: bool,

    /// Search in project config
    #[arg(long)]
    project: bool,

    /// Show detailed output
    #[arg(long)]
    verbose: bool,
}

impl SearchArgs {
    /// Execute the search command
    pub fn execute(&self) -> Result<()> {
        let backup_dir = claude_config_manager_core::paths::get_backup_dir();
        let manager = ConfigManager::new(&backup_dir);

        // Build search options
        let mut options = SearchOptions::new()
            .with_case_sensitive(self.case_sensitive)
            .with_max_depth(self.depth);

        if self.value {
            options = options.with_keys(false).with_values(true);
        } else if self.both {
            options = options.with_keys(true).with_values(true);
        }
        // default is keys only

        // Determine scope
        let scope = if self.global {
            ConfigScope::Global
        } else if self.project {
            ConfigScope::Project
        } else {
            // Default: try project first, then global
            ConfigScope::Project
        };

        // Perform search
        let results = manager.search_config_with_options(&self.query, scope, options)?;

        // Display results
        if results.is_empty() {
            println!("No matches found for '{}'", self.query);
            return Ok(());
        }

        println!("Found {} result(s) for '{}':\n", results.len(), self.query);

        for (index, result) in results.iter().enumerate() {
            if self.verbose {
                println!("  [{}] {}", index + 1, result.format());
                println!("      Type: {}", result.value_type_label());
                println!("      Config: {}", result.config_path.display());
            } else {
                println!("  [{}] {}", index + 1, result.format());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_args_builder() {
        let args = SearchArgs {
            query: "test".to_string(),
            value: false,
            both: false,
            case_sensitive: true,
            depth: Some(5),
            global: true,
            project: false,
            verbose: false,
        };

        assert_eq!(args.query, "test");
        assert!(args.case_sensitive);
        assert_eq!(args.depth, Some(5));
        assert!(args.global);
    }
}
