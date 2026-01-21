//! Project management commands
//!
//! Implements `project scan` and `project list` commands for discovering
//! and managing Claude Code projects.

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use claude_config_manager_core::{ConfigManager, ProjectScanner};

/// Project management command arguments
#[derive(Parser, Debug)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

/// Project management commands
#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// Scan directory for Claude Code projects
    Scan {
        /// Directory path to scan (default: current directory)
        #[arg(short, long)]
        path: Option<Utf8PathBuf>,

        /// Maximum scan depth (default: unlimited)
        #[arg(short, long)]
        depth: Option<usize>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// List discovered projects
    List {
        /// Directory path to scan (default: current directory)
        #[arg(short, long)]
        path: Option<Utf8PathBuf>,

        /// Maximum scan depth (default: unlimited)
        #[arg(short, long)]
        depth: Option<usize>,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show configuration for a project
    Config {
        /// Project path
        path: Utf8PathBuf,
    },
}

impl ProjectCommand {
    /// Execute the project command
    pub fn execute(&self) -> Result<()> {
        match self {
            ProjectCommand::Scan {
                path,
                depth,
                verbose,
            } => self.scan(path.as_deref(), *depth, *verbose),
            ProjectCommand::List {
                path,
                depth,
                verbose,
            } => self.list(path.as_deref(), *depth, *verbose),
            ProjectCommand::Config { path } => self.show_config(path),
        }
    }

    /// Scan directory for projects
    fn scan(
        &self,
        path: Option<&camino::Utf8Path>,
        depth: Option<usize>,
        verbose: bool,
    ) -> Result<()> {
        let scan_path = if let Some(p) = path {
            p
        } else {
            camino::Utf8Path::new(".")
        };
        let scanner = ProjectScanner::new(depth, false);

        println!("Scanning for Claude Code projects in: {scan_path}\n");

        let start = std::time::Instant::now();
        let projects = scanner.scan_directory(scan_path.as_ref())?;
        let duration = start.elapsed();

        if projects.is_empty() {
            println!("No projects found.");
            return Ok(());
        }

        println!("Found {} project(s):\n", projects.len());

        for (index, project) in projects.iter().enumerate() {
            println!("  [{}] {}", index + 1, project.name);

            if verbose {
                println!("      Root: {}", project.root.display());
                println!("      Claude: {}", project.claude_dir.display());
                println!("      Config: {}", project.config_path.display());
                println!("      Has Config: {}", project.has_config);

                if let Some(modified) = project.last_modified {
                    let duration_since = modified.elapsed().unwrap_or_default().as_secs();
                    println!("      Last Modified: {duration_since} seconds ago");
                }
            } else {
                println!("      {}", project.root.display());
            }
            println!();
        }

        println!("Scan completed in {duration:?}");

        Ok(())
    }

    /// List discovered projects
    fn list(
        &self,
        path: Option<&camino::Utf8Path>,
        depth: Option<usize>,
        verbose: bool,
    ) -> Result<()> {
        let scan_path = if let Some(p) = path {
            p
        } else {
            camino::Utf8Path::new(".")
        };
        let scanner = ProjectScanner::new(depth, false);

        let projects = scanner.scan_directory(scan_path.as_ref())?;

        if projects.is_empty() {
            println!("No projects found.");
            return Ok(());
        }

        // Format as table
        println!("Claude Code Projects ({}):\n", projects.len());

        for (index, project) in projects.iter().enumerate() {
            println!("  [{}]  {}", index + 1, project.name);

            if verbose {
                println!("       Path: {}", project.root.display());
                println!("       Config: {}", project.config_path.display());

                if let Some(modified) = project.last_modified {
                    if let Ok(duration) = modified.elapsed() {
                        let duration_str = if duration.as_secs() < 60 {
                            format!("{}s ago", duration.as_secs())
                        } else {
                            format!("{}m ago", duration.as_secs() / 60)
                        };
                        println!("       Modified: {duration_str}");
                    }
                }
            }
        }

        println!("\nUse 'ccm project config <path>' to view project configuration");

        Ok(())
    }

    /// Show configuration for a specific project
    fn show_config(&self, path: &camino::Utf8Path) -> Result<()> {
        let backup_dir = claude_config_manager_core::paths::get_backup_dir();
        let manager = ConfigManager::new(&backup_dir);

        // Read project config
        let config_path = path.join(".claude").join("config.json");
        let config = manager.read_config(config_path.as_ref())?;

        // Display configuration
        println!("Project Configuration: {path}\n");

        // Show MCP servers
        if let Some(servers) = config.mcp_servers {
            if !servers.is_empty() {
                println!("MCP Servers:");
                for (name, server) in servers.iter() {
                    println!("  {name}:");
                    println!("    Enabled: {}", server.enabled);
                    if let Some(cmd) = &server.command {
                        println!("    Command: {cmd}");
                    }
                    if !server.args.is_empty() {
                        println!("    Args: {}", server.args.join(" "));
                    }
                }
                println!();
            }
        }

        // Show custom instructions
        if let Some(instructions) = config.custom_instructions {
            if !instructions.is_empty() {
                println!("Custom Instructions:");
                for (i, instruction) in instructions.iter().enumerate() {
                    println!("  {}. {}", i + 1, instruction);
                }
                println!();
            }
        }

        // Show allowed paths
        if let Some(paths) = config.allowed_paths {
            if !paths.is_empty() {
                println!("Allowed Paths:");
                for path_item in paths.iter() {
                    println!("  - {path_item}");
                }
                println!();
            }
        }

        // Show skills
        if let Some(skills) = config.skills {
            if !skills.is_empty() {
                println!("Enabled Skills:");
                for (skill_name, _skill) in skills.iter() {
                    println!("  - {skill_name}");
                }
                println!();
            }
        }

        Ok(())
    }
}
