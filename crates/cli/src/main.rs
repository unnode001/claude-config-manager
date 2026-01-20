//! Claude Config Manager - CLI Application
//!
//! Command-line interface for managing Claude Code configuration files.

use clap::Parser;

mod commands;
mod key_path;
mod output;

use commands::config::ConfigArgs;
use commands::history::HistoryArgs;
use commands::mcp::McpArgs;
use commands::project::ProjectArgs;
use commands::search::SearchArgs;

/// Claude Config Manager - Manage Claude Code configurations
#[derive(Parser, Debug)]
#[command(name = "ccm")]
#[command(author = "Claude Config Manager Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A centralized configuration management tool for Claude Code", long_about = None)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Configuration management commands
    Config(ConfigArgs),
    /// History and backup management commands
    History(HistoryArgs),
    /// MCP server management commands
    Mcp(McpArgs),
    /// Project discovery and management commands
    Project(ProjectArgs),
    /// Search configuration values
    Search(SearchArgs),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(log_level).init();

    tracing::debug!("Claude Config Manager v{}", env!("CARGO_PKG_VERSION"));

    // Execute command
    match args.command {
        Some(Commands::Config(config_args)) => {
            config_args.execute()?;
        }
        Some(Commands::History(history_args)) => {
            history_args.execute()?;
        }
        Some(Commands::Mcp(mcp_args)) => {
            mcp_args.execute()?;
        }
        Some(Commands::Project(project_args)) => {
            project_args.command.execute()?;
        }
        Some(Commands::Search(search_args)) => {
            search_args.execute()?;
        }
        None => {
            println!("Claude Config Manager v{}", env!("CARGO_PKG_VERSION"));
            println!("\nUsage: ccm <command> [options]");
            println!("\nCommands:");
            println!("  config      Configuration management");
            println!("  history     Backup and history management");
            println!("  mcp         MCP server management");
            println!("  project     Project discovery and management");
            println!("  search      Search configuration values");
            println!("\nRun 'ccm help <command>' for more information.");
        }
    }

    Ok(())
}
