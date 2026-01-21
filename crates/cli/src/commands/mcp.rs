//! MCP Server management commands
//!
//! Implements `mcp list`, `mcp enable`, `mcp disable`, `mcp add`, `mcp remove`, and `mcp show` commands

use anyhow::Result;
use clap::Parser;
use claude_config_manager_core::{ConfigScope, McpManager, McpServer};
use std::path::{Path, PathBuf};

/// MCP server management commands
#[derive(Parser, Debug)]
pub struct McpArgs {
    /// Project path (default: auto-detect)
    #[arg(short, long)]
    project: Option<PathBuf>,

    /// Configuration scope (global or project)
    #[arg(short, long, default_value = "global")]
    scope: String,

    #[command(subcommand)]
    command: McpCommand,
}

/// MCP subcommands
#[derive(Parser, Debug)]
enum McpCommand {
    /// List all MCP servers
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Enable an MCP server
    Enable {
        /// Server name
        name: String,
    },
    /// Disable an MCP server
    Disable {
        /// Server name
        name: String,
    },
    /// Add a new MCP server
    Add {
        /// Server name
        name: String,
        /// Command to run (e.g., "npx", "uvx")
        #[arg(short, long)]
        command: String,
        /// Arguments to pass to the command
        #[arg(short, long, default_value = "")]
        args: String,
        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },
    /// Remove an MCP server
    Remove {
        /// Server name
        name: String,
    },
    /// Show detailed server information
    Show {
        /// Server name
        name: String,
    },
}

impl McpArgs {
    /// Execute the MCP command
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            McpCommand::List { verbose } => {
                self.cmd_list(*verbose)?;
            }
            McpCommand::Enable { name } => {
                self.cmd_enable(name)?;
            }
            McpCommand::Disable { name } => {
                self.cmd_disable(name)?;
            }
            McpCommand::Add {
                name,
                command,
                args,
                env,
            } => {
                self.cmd_add(name, command, args, env)?;
            }
            McpCommand::Remove { name } => {
                self.cmd_remove(name)?;
            }
            McpCommand::Show { name } => {
                self.cmd_show(name)?;
            }
        }
        Ok(())
    }

    /// Parse scope from string
    fn parse_scope(&self) -> Result<ConfigScope> {
        match self.scope.to_lowercase().as_str() {
            "global" => Ok(ConfigScope::Global),
            "project" => Ok(ConfigScope::Project),
            _ => anyhow::bail!("Invalid scope '{}'. Use 'global' or 'project'.", self.scope),
        }
    }

    /// Get project path for the command
    fn get_project_path(&self) -> Option<&Path> {
        self.project.as_deref()
    }

    /// Create backup directory path
    fn get_backup_dir() -> PathBuf {
        // Use a simple relative path for backups
        PathBuf::from(".backups")
    }

    /// List MCP servers
    fn cmd_list(&self, verbose: bool) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        let manager = McpManager::new(&backup_dir);
        let servers = manager.list_servers(&scope, project_path)?;

        if servers.is_empty() {
            println!("No MCP servers configured.");
            return Ok(());
        }

        println!("MCP Servers ({}):\n", servers.len());

        for (name, server) in servers.iter() {
            println!("  {name}:");
            println!("    Enabled: {}", if server.enabled { "yes" } else { "no" });
            println!(
                "    Command: {}",
                server.command.as_deref().unwrap_or("(default)")
            );

            if !server.args.is_empty() {
                println!("    Args: {}", server.args.join(" "));
            }

            if !server.env.is_empty() {
                println!("    Env:");
                for (key, value) in &server.env {
                    println!("      {key}={value}");
                }
            }

            if verbose {
                println!("    Name: {}", server.name);
            }

            println!();
        }

        Ok(())
    }

    /// Enable an MCP server
    fn cmd_enable(&self, name: &str) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        let manager = McpManager::new(&backup_dir);
        manager.enable_server(name, &scope, project_path)?;

        println!("MCP server '{name}' enabled successfully.");
        Ok(())
    }

    /// Disable an MCP server
    fn cmd_disable(&self, name: &str) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        let manager = McpManager::new(&backup_dir);
        manager.disable_server(name, &scope, project_path)?;

        println!("MCP server '{name}' disabled successfully.");
        Ok(())
    }

    /// Add a new MCP server
    fn cmd_add(&self, name: &str, command: &str, args: &str, env_vars: &[String]) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        // Parse arguments
        let args_vec: Vec<String> = if args.is_empty() {
            vec![]
        } else {
            args.split(' ').map(|s| s.to_string()).collect()
        };

        // Parse environment variables
        let mut env_map = std::collections::HashMap::new();
        for env_var in env_vars {
            let parts: Vec<&str> = env_var.splitn(2, '=').collect();
            if parts.len() == 2 {
                env_map.insert(parts[0].to_string(), parts[1].to_string());
            }
        }

        // Create server
        let mut server = McpServer::new(name, command, args_vec);
        server.env = env_map;

        let manager = McpManager::new(&backup_dir);
        manager.add_server(name, server, &scope, project_path)?;

        println!("MCP server '{name}' added successfully.");
        Ok(())
    }

    /// Remove an MCP server
    fn cmd_remove(&self, name: &str) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        let manager = McpManager::new(&backup_dir);
        manager.remove_server(name, &scope, project_path)?;

        println!("MCP server '{name}' removed successfully.");
        Ok(())
    }

    /// Show detailed server information
    fn cmd_show(&self, name: &str) -> Result<()> {
        let scope = self.parse_scope()?;
        let project_path = self.get_project_path();
        let backup_dir = Self::get_backup_dir();

        let manager = McpManager::new(&backup_dir);
        let server = manager.get_server(name, &scope, project_path)?;

        println!("Server: {name}");
        println!("  Enabled: {}", if server.enabled { "yes" } else { "no" });
        println!(
            "  Command: {}",
            server.command.as_deref().unwrap_or("(default)")
        );

        let args_str = if server.args.is_empty() {
            "(none)".to_string()
        } else {
            server.args.join(" ")
        };
        println!("  Args: {args_str}");

        let env_str = if server.env.is_empty() {
            "(none)".to_string()
        } else {
            server
                .env
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        println!("  Environment: {env_str}");

        Ok(())
    }
}
