//! CLI Integration Tests
//!
//! Tests the CLI commands end-to-end using assert_cmd.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
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
            .stdout(predicate::str::contains("A centralized configuration management tool"));
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

    // Additional integration tests will be added as CLI features evolve
    // These tests verify the basic CLI structure and command availability
}
