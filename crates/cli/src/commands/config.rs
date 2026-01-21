//! Configuration management commands
//!
//! Implements `config get` and `config set` commands

use crate::key_path::set_value_by_path;
use crate::output::{format_json, format_table};
use anyhow::Result;
use clap::Parser;
use claude_config_manager_core::{
    paths::get_global_config_path, ConfigDiff, ConfigManager, ConfigScope,
};
use std::path::PathBuf;

/// Configuration management commands
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// Project path (default: auto-detect)
    #[arg(short, long)]
    project: Option<PathBuf>,

    /// Output format
    #[arg(short, long, default_value = "table")]
    output: OutputFormat,

    #[command(subcommand)]
    command: ConfigCommand,
}

/// Output format for configuration display
#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum OutputFormat {
    /// Human-readable table format
    Table,
    /// Machine-readable JSON format
    Json,
}

/// Configuration subcommands
#[derive(Parser, Debug)]
enum ConfigCommand {
    /// Get configuration value(s)
    Get {
        /// Configuration key (e.g., "mcpServers.npx.enabled")
        /// If omitted, shows all configuration
        key: Option<String>,
    },
    /// Set configuration value
    Set {
        /// Configuration key (e.g., "mcpServers.npx.enabled")
        key: String,
        /// Configuration value (JSON for objects/arrays)
        value: String,
    },
    /// Show differences between global and project configuration
    Diff {
        /// Project path (default: auto-detect if not provided via --project flag)
        project_path: Option<PathBuf>,
    },
    /// Export configuration to a file
    Export {
        /// Output file path
        output_file: PathBuf,
    },
    /// Import configuration from a file
    Import {
        /// Input file path
        input_file: PathBuf,
        /// Skip validation
        #[arg(long)]
        no_validate: bool,
    },
}

impl ConfigArgs {
    /// Execute the configuration command
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            ConfigCommand::Get { key } => {
                self.cmd_get(key.as_deref())?;
            }
            ConfigCommand::Set { key, value } => {
                self.cmd_set(key, value)?;
            }
            ConfigCommand::Diff { project_path } => {
                self.cmd_diff(project_path.as_ref())?;
            }
            ConfigCommand::Export { output_file } => {
                self.cmd_export(output_file)?;
            }
            ConfigCommand::Import {
                input_file,
                no_validate,
            } => {
                self.cmd_import(input_file, !no_validate)?;
            }
        }
        Ok(())
    }

    /// Get configuration value(s)
    fn cmd_get(&self, key: Option<&str>) -> Result<()> {
        // Create backup directory (use global config dir for backups)
        let backup_dir = get_global_config_path()
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from(".backups"));

        let manager = ConfigManager::new(&backup_dir);

        // Get configuration
        let config = if let Some(project_path) = &self.project {
            manager.get_merged_config(Some(project_path))?
        } else {
            manager.get_merged_config(None)?
        };

        // Output based on format
        match self.output {
            OutputFormat::Json => {
                format_json(&config, key)?;
            }
            OutputFormat::Table => {
                format_table(&config, key)?;
            }
        }

        Ok(())
    }

    /// Set configuration value
    fn cmd_set(&self, key: &str, value: &str) -> Result<()> {
        // Determine which config file to modify
        let config_path = if let Some(project_path) = &self.project {
            project_path.join(".claude").join("config.json")
        } else {
            get_global_config_path()
        };

        let backup_dir = config_path
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from(".backups"));

        let manager = ConfigManager::new(&backup_dir);

        // Read existing config or create new one
        let mut config = if config_path.exists() {
            manager.read_config(&config_path)?
        } else {
            claude_config_manager_core::ClaudeConfig::new()
        };

        // Set the value using key path
        set_value_by_path(&mut config, key, value)?;

        // Write config with backup
        manager.write_config_with_backup(&config_path, &config)?;

        // Success message
        if config_path.exists() {
            println!("Configuration updated successfully.");
            println!(
                "Backup created at: {:?}",
                manager.backup_manager().list_backups(&config_path)?.last()
            );
        }

        Ok(())
    }

    /// Show configuration differences
    fn cmd_diff(&self, project_path: Option<&PathBuf>) -> Result<()> {
        // Create backup directory
        let backup_dir = get_global_config_path()
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from(".backups"));

        let manager = ConfigManager::new(&backup_dir);

        // Determine project path
        let project = if let Some(p) = project_path {
            p.as_path()
        } else if let Some(p) = &self.project {
            p.as_path()
        } else {
            std::path::Path::new(".")
        };

        // Get diffs
        let (diffs, source_map) = manager.diff_configs(Some(project))?;

        // Display results
        if diffs.is_empty() {
            println!("No differences found between global and project configuration.");
            return Ok(());
        }

        println!("Configuration differences ({} total):\n", diffs.len());

        // Group diffs by type
        let mut additions = Vec::new();
        let mut removals = Vec::new();
        let mut modifications = Vec::new();

        for diff in &diffs {
            match diff {
                ConfigDiff::Added { .. } => additions.push(diff),
                ConfigDiff::Removed { .. } => removals.push(diff),
                ConfigDiff::Modified { .. } => modifications.push(diff),
            }
        }

        // Display additions (green)
        if !additions.is_empty() {
            println!("Additions (project-specific):");
            for diff in additions {
                if let ConfigDiff::Added { key_path, value } = diff {
                    println!("  + {key_path}");
                    if matches!(self.output, OutputFormat::Json) {
                        println!("    {}", serde_json::to_string_pretty(value)?);
                    }
                }
            }
            println!();
        }

        // Display removals (red)
        if !removals.is_empty() {
            println!("Removals (missing in project):");
            for diff in removals {
                if let ConfigDiff::Removed { key_path, .. } = diff {
                    println!("  - {key_path}");
                }
            }
            println!();
        }

        // Display modifications (yellow)
        if !modifications.is_empty() {
            println!("Modifications (different values):");
            for diff in modifications {
                if let ConfigDiff::Modified {
                    key_path,
                    old_value,
                    new_value,
                } = diff
                {
                    println!("  ~ {key_path}");
                    if matches!(self.output, OutputFormat::Json) {
                        println!("    old: {}", serde_json::to_string_pretty(old_value)?);
                        println!("    new: {}", serde_json::to_string_pretty(new_value)?);
                    }
                }
            }
            println!();
        }

        // Display source summary
        println!("Source summary:");
        let mut global_count = 0;
        let mut project_count = 0;
        for scope in source_map.sources.values() {
            match scope {
                ConfigScope::Global => global_count += 1,
                ConfigScope::Project => project_count += 1,
            }
        }
        println!("  Values from global: {global_count}");
        println!("  Values from project: {project_count}");

        Ok(())
    }

    /// Export configuration to a file
    fn cmd_export(&self, output_file: &PathBuf) -> Result<()> {
        let backup_dir = get_global_config_path()
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from(".backups"));

        let manager = ConfigManager::new(&backup_dir);

        // Get configuration to export
        let config = if let Some(project_path) = &self.project {
            manager.get_merged_config(Some(project_path))?
        } else {
            manager.get_merged_config(None)?
        };

        // Export configuration
        let exported_path = manager.export_config(&config, output_file)?;

        println!("Configuration exported to: {}", exported_path.display());

        Ok(())
    }

    /// Import configuration from a file
    fn cmd_import(&self, input_file: &PathBuf, validate: bool) -> Result<()> {
        let backup_dir = get_global_config_path()
            .parent()
            .map(|p| p.join("backups"))
            .unwrap_or_else(|| PathBuf::from(".backups"));

        let manager = ConfigManager::new(&backup_dir);

        // Import configuration
        let mut options = claude_config_manager_core::ImportExportOptions::default();
        options.validate = validate;

        let imported_config = manager.import_config_with_options(input_file, options)?;

        // Determine target path
        let target_path = if let Some(project_path) = &self.project {
            project_path.join(".claude").join("config.json")
        } else {
            get_global_config_path()
        };

        // Write imported configuration
        manager.write_config_with_backup(&target_path, &imported_config)?;

        println!("Configuration imported from: {}", input_file.display());
        println!("Written to: {}", target_path.display());

        Ok(())
    }
}
