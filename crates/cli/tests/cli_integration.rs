//! CLI Integration Tests
//!
//! Tests the CLI commands end-to-end using assert_cmd.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper struct to set up and tear down test environment
struct TestEnv {
    temp_dir: TempDir,
    config_path: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        Self {
            temp_dir,
            config_path,
        }
    }

    /// Create a config file with test data
    fn create_test_config(&self) {
        let test_config = r#"{
            "customInstructions": "Test instructions",
            "mcpServers": {
                "test-server": {
                    "command": "npx",
                    "args": ["-y"]
                }
            }
        }"#;
        fs::write(&self.config_path, test_config).unwrap();
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_cli_version() {
        Command::cargo_bin("ccm")
            .unwrap()
            .arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::contains("ccm"));
    }

    #[test]
    fn test_cli_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .arg("--help")
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "A centralized configuration management tool",
            ));
    }

    #[test]
    fn test_config_subcommand_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Configuration management"));
    }

    #[test]
    fn test_mcp_subcommand_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["mcp", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("MCP server management"));
    }

    #[test]
    fn test_env_setup() {
        let env = TestEnv::new();
        assert!(env.temp_dir.path().exists());
    }

    // Note: Tests that modify the actual global config require special handling
    // to avoid interfering with the user's actual configuration.
    // Future tests should use custom config paths via environment variables or CLI flags.

    #[test]
    fn test_config_get_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "get", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_config_set_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "set", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_config_diff_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "diff", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_mcp_list_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["mcp", "list", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_mcp_add_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["mcp", "add", "--help"])
            .assert()
            .success();
    }

    #[test]
    fn test_project_subcommand_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["project", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Project discovery and management"));
    }

    #[test]
    fn test_project_scan_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["project", "scan", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Scan directory"));
    }

    #[test]
    fn test_project_list_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["project", "list", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("List discovered"));
    }

    #[test]
    fn test_project_config_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["project", "config", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Show configuration"));
    }

    #[test]
    fn test_project_scan_no_projects() {
        let temp_dir = TempDir::new().unwrap();

        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "project",
                "scan",
                "--path",
                temp_dir.path().to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("No projects found"));
    }

    #[test]
    fn test_project_scan_finds_projects() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test project
        let project_dir = temp_dir.path().join("test-project");
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("config.json"), r#"{"mcpServers": {}}"#).unwrap();

        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "project",
                "scan",
                "--path",
                temp_dir.path().to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Found 1 project"))
            .stdout(predicate::str::contains("test-project"));
    }

    #[test]
    fn test_project_scan_verbose() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test project
        let project_dir = temp_dir.path().join("verbose-project");
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("config.json"), "{}").unwrap();

        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "project",
                "scan",
                "--path",
                temp_dir.path().to_str().unwrap(),
                "--verbose",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("Root:"))
            .stdout(predicate::str::contains("Claude:"))
            .stdout(predicate::str::contains("Config:"))
            .stdout(predicate::str::contains("Has Config:"));
    }

    #[test]
    fn test_project_list_no_projects() {
        let temp_dir = TempDir::new().unwrap();

        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "project",
                "list",
                "--path",
                temp_dir.path().to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("No projects found"));
    }

    #[test]
    fn test_project_scan_respects_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested project structure
        let level1 = temp_dir.path().join("level1");
        let level2 = level1.join("level2");
        let level3_project = level2.join("deep-project");
        let claude_dir = level3_project.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("config.json"), "{}").unwrap();

        // Scan with depth 1 should not find the deep project
        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "project",
                "scan",
                "--path",
                temp_dir.path().to_str().unwrap(),
                "--depth",
                "1",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("No projects found"));
    }

    #[test]
    fn test_history_list_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["history", "list", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("List available"));
    }

    #[test]
    fn test_history_restore_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["history", "restore", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Restore a backup"));
    }

    #[test]
    fn test_history_list_empty() {
        let temp_dir = TempDir::new().unwrap();

        Command::cargo_bin("ccm")
            .unwrap()
            .args([
                "history",
                "list",
                "--project",
                temp_dir.path().to_str().unwrap(),
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("No backups found"));
    }

    #[test]
    fn test_search_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["search", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Search configuration"));
    }

    #[test]
    fn test_config_export_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "export", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Export configuration"));
    }

    #[test]
    fn test_config_import_help() {
        Command::cargo_bin("ccm")
            .unwrap()
            .args(["config", "import", "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Import configuration"));
    }

    // Additional integration tests will be added as CLI features evolve
    // These tests verify the basic CLI structure and command availability
}
