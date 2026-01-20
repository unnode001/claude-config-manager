//! History command implementation
//!
//! Provides backup listing and restoration functionality

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use claude_config_manager_core::{
    backup::BackupManager,
    paths::get_backup_dir,
};
use std::path::PathBuf;

/// History management commands
#[derive(Parser, Debug)]
pub struct HistoryArgs {
    #[command(subcommand)]
    command: HistoryCommand,
}

/// History management commands
#[derive(Subcommand, Debug)]
pub enum HistoryCommand {
    /// List available backups
    List {
        /// Show detailed information about each backup
        #[arg(short, long)]
        verbose: bool,

        /// Maximum number of backups to display
        #[arg(short, long)]
        limit: Option<usize>,

        /// Project path (for project-specific backups)
        #[arg(short, long)]
        project: Option<Utf8PathBuf>,
    },

    /// Restore a backup
    Restore {
        /// Backup file path or index (from list command)
        backup: String,

        /// Project path (for project-specific backups)
        #[arg(short, long)]
        project: Option<Utf8PathBuf>,

        /// Don't ask for confirmation before restoring
        #[arg(short, long)]
        yes: bool,
    },
}

impl HistoryArgs {
    /// Execute the history command
    pub fn execute(&self) -> Result<()> {
        self.command.execute()
    }
}

impl HistoryCommand {
    /// Execute the history command
    pub fn execute(&self) -> Result<()> {
        match self {
            HistoryCommand::List { verbose, limit, project } => {
                self.list_backups(*verbose, *limit, project.as_deref())
            }
            HistoryCommand::Restore { backup, project, yes } => {
                self.restore_backup(backup, project.as_deref(), *yes)
            }
        }
    }

    /// List available backups
    fn list_backups(&self, verbose: bool, limit: Option<usize>, project_path: Option<&camino::Utf8Path>) -> Result<()> {
        // Determine backup directory
        let backup_dir = if let Some(project) = project_path {
            get_backup_dir().join(project.join(".claude"))
        } else {
            get_backup_dir()
        };

        let manager = BackupManager::new(&backup_dir, None);

        // Determine the original config file path
        let original_file: PathBuf = if let Some(project) = project_path {
            project.join(".claude").join("config.json").into_std_path_buf()
        } else {
            // Global config is in parent of backup dir
            backup_dir
                .parent()
                .unwrap_or(&backup_dir)
                .join("config.json")
        };

        let backups = manager.list_backups(original_file.as_ref())?;

        if backups.is_empty() {
            println!("No backups found.");
            return Ok(());
        }

        let total_count = backups.len();

        // Apply limit if specified
        let backups_to_show: Vec<_> = if let Some(limit) = limit {
            backups.into_iter().take(limit).collect()
        } else {
            backups.into_iter().collect()
        };

        println!("Backups ({} available, showing {}):\n", total_count, backups_to_show.len());

        for (index, backup) in backups_to_show.iter().enumerate() {
            // Print index for easy reference
            println!("  [{}]  {}", index, backup_path_display(&backup.path));

            if verbose {
                println!("       Created: {}", format_timestamp(&backup.created_at));
                println!("       Size: {} bytes", backup.size);
                println!("       Original: {}", backup.original_path);
            } else {
                println!("       Created: {}", format_timestamp(&backup.created_at));
            }
            println!();
        }

        println!("Use 'ccm history restore <index or path>' to restore a backup");

        Ok(())
    }

    /// Restore a backup
    fn restore_backup(&self, backup_spec: &str, project_path: Option<&camino::Utf8Path>, yes: bool) -> Result<()> {
        // Determine backup directory
        let backup_dir = if let Some(project) = project_path {
            get_backup_dir().join(project.join(".claude"))
        } else {
            get_backup_dir()
        };

        let manager = BackupManager::new(&backup_dir, None);

        // Determine the original config file path
        let original_file: PathBuf = if let Some(project) = project_path {
            project.join(".claude").join("config.json").into_std_path_buf()
        } else {
            backup_dir
                .parent()
                .unwrap_or(&backup_dir)
                .join("config.json")
        };

        // Parse backup_spec as either index or path
        let backup_path = if let Ok(index) = backup_spec.parse::<usize>() {
            // It's an index - list backups and get the one at this index
            let backups = manager.list_backups(original_file.as_ref())?;

            if index >= backups.len() {
                anyhow::bail!("Invalid backup index: {}. Only {} backups available.",
                    index, backups.len());
            }

            std::path::PathBuf::from(&backups[index].path)
        } else {
            // It's a path - use it directly
            std::path::PathBuf::from(backup_spec)
        };

        // Verify backup exists
        if !backup_path.exists() {
            anyhow::bail!("Backup not found: {}", backup_path.display());
        }

        // Show what will be restored
        println!("Backup to restore: {}", backup_path.display());
        println!("Target file: {}", original_file.as_path().display());
        println!();

        // Ask for confirmation unless --yes was specified
        if !yes {
            print!("Are you sure you want to restore this backup? [y/N] ");
            use std::io::Write;
            std::io::stdout().flush()?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                println!("Restore cancelled.");
                return Ok(());
            }
        }

        // Restore the backup
        let restored_path = manager.restore_backup(&backup_path)?;

        println!("✓ Backup restored successfully: {}", restored_path.display());

        Ok(())
    }
}

/// Format backup path for display (shorten if needed)
fn backup_path_display(path: &str) -> String {
    let path = std::path::Path::new(path);

    // Extract just the filename if it's in the backup directory
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        file_name.to_string()
    } else {
        path.display().to_string()
    }
}

/// Format timestamp for display
fn format_timestamp(dt: &chrono::DateTime<chrono::Utc>) -> String {
    // Format as: 2025-01-20 14:30:45
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
