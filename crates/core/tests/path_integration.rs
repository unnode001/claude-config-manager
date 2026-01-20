//! Integration tests for path resolution and project detection
//!
//! These tests verify real-world filesystem operations for configuration path handling.

use claude_config_manager_core::{
    expand_tilde, find_project_config, get_global_config_dir, get_global_config_path,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[test]
fn test_global_config_path_is_consistent() {
    let config_dir = get_global_config_dir();
    let config_path = get_global_config_path();

    // Config path should start with config dir
    assert!(config_path.starts_with(&config_dir));

    // Config path should be config_dir + config.json
    assert_eq!(
        config_path.strip_prefix(&config_dir).unwrap(),
        Path::new("config.json")
    );
}

#[test]
fn test_find_project_config_in_real_directory_structure() {
    let temp_dir = TempDir::new().unwrap();

    // Create realistic directory structure
    let project_root = temp_dir.path().join("my-project");
    let src_dir = project_root.join("src");
    let nested_src = src_dir.join("components");

    fs::create_dir_all(&nested_src).unwrap();

    // Create .claude directory at project root
    let claude_dir = project_root.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Create config file
    let config_path = claude_dir.join("config.json");
    let config_content = r#"{
        "mcpServers": {
            "npx": {
                "enabled": true
            }
        }
    }"#;
    fs::write(&config_path, config_content).unwrap();

    // Start search from deeply nested directory
    let found = find_project_config(Some(&nested_src));

    assert!(found.is_some());
    assert_eq!(found.unwrap(), config_path);
}

#[test]
fn test_find_project_config_prefers_closest_config() {
    let temp_dir = TempDir::new().unwrap();

    // Create outer project with config
    let outer_project = temp_dir.path().join("outer");
    fs::create_dir_all(&outer_project).unwrap();
    let outer_claude = outer_project.join(".claude");
    fs::create_dir_all(&outer_claude).unwrap();
    let outer_config = outer_claude.join("config.json");
    fs::write(&outer_config, r#"{"name": "outer"}"#).unwrap();

    // Create inner project with config
    let inner_project = outer_project.join("inner");
    fs::create_dir_all(&inner_project).unwrap();
    let inner_claude = inner_project.join(".claude");
    fs::create_dir_all(&inner_claude).unwrap();
    let inner_config = inner_claude.join("config.json");
    fs::write(&inner_config, r#"{"name": "inner"}"#).unwrap();

    // Start from inner project - should find inner config
    let found = find_project_config(Some(&inner_project));

    assert!(found.is_some());
    assert_eq!(found.unwrap(), inner_config);
}

#[test]
fn test_find_project_config_stops_at_git_boundary() {
    let temp_dir = TempDir::new().unwrap();

    // Create Git repository root
    let git_root = temp_dir.path().join("git-repo");
    fs::create_dir_all(&git_root).unwrap();
    fs::create_dir_all(git_root.join(".git")).unwrap();

    // Create config ABOVE Git root
    let above_git = temp_dir.path().join(".claude");
    fs::create_dir_all(&above_git).unwrap();
    let above_config = above_git.join("config.json");
    fs::write(&above_config, r#"{"above": true}"#).unwrap();

    // Create nested directory inside Git repo
    let nested = git_root.join("nested").join("deep");
    fs::create_dir_all(&nested).unwrap();

    // Start from nested directory - should NOT find config above Git root
    let found = find_project_config(Some(&nested));

    assert!(found.is_none());
}

#[test]
fn test_expand_tilde_with_real_home_directory() {
    if let Some(home) = dirs::home_dir() {
        let tilde_path = PathBuf::from("~/test/path");
        let expanded = expand_tilde(&tilde_path);

        // Should start with home directory
        assert!(expanded.starts_with(&home));

        // Should not contain tilde
        let path_str = expanded.to_string_lossy();
        assert!(!path_str.starts_with('~'));
    }
}

#[test]
fn test_find_project_config_with_symlink_like_structure() {
    let temp_dir = TempDir::new().unwrap();

    // Create directory structure that mimics symlink scenarios
    let project = temp_dir.path().join("project");
    let level1 = project.join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");

    fs::create_dir_all(&level3).unwrap();

    // Place config at level1
    let claude_dir = level1.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();
    let config_path = claude_dir.join("config.json");
    fs::write(&config_path, r#"{"level": 1}"#).unwrap();

    // Start from level3 - should find config at level1
    let found = find_project_config(Some(&level3));

    assert!(found.is_some());
    assert_eq!(found.unwrap(), config_path);
}

#[test]
fn test_multiple_nested_projects() {
    let temp_dir = TempDir::new().unwrap();

    // Create monorepo structure
    let monorepo = temp_dir.path().join("monorepo");
    let frontend = monorepo.join("frontend");
    let backend = monorepo.join("backend");

    fs::create_dir_all(&frontend).unwrap();
    fs::create_dir_all(&backend).unwrap();

    // Each subproject has its own config
    let frontend_claude = frontend.join(".claude");
    fs::create_dir_all(&frontend_claude).unwrap();
    let frontend_config = frontend_claude.join("config.json");
    fs::write(&frontend_config, r#"{"project": "frontend"}"#).unwrap();

    let backend_claude = backend.join(".claude");
    fs::create_dir_all(&backend_claude).unwrap();
    let backend_config = backend_claude.join("config.json");
    fs::write(&backend_config, r#"{"project": "backend"}"#).unwrap();

    // Find frontend config from frontend directory
    let found_frontend = find_project_config(Some(&frontend));
    assert!(found_frontend.is_some());
    assert_eq!(found_frontend.unwrap(), frontend_config);

    // Find backend config from backend directory
    let found_backend = find_project_config(Some(&backend));
    assert!(found_backend.is_some());
    assert_eq!(found_backend.unwrap(), backend_config);
}

#[test]
fn test_expand_tilde_in_nested_path() {
    if let Some(home) = dirs::home_dir() {
        let tilde_path = PathBuf::from("~/projects/rust/project");
        let expanded = expand_tilde(&tilde_path);

        // Should preserve full path structure
        let path_str = expanded.to_string_lossy();
        assert!(path_str.contains("projects"));
        assert!(path_str.contains("rust"));
        assert!(path_str.contains("project"));

        // Should start with home
        assert!(expanded.starts_with(&home));
    }
}

#[test]
fn test_empty_craude_directory_handling() {
    let temp_dir = TempDir::new().unwrap();

    let project = temp_dir.path().join("project");
    fs::create_dir_all(&project).unwrap();

    // Create .claude directory but no config file
    let claude_dir = project.join(".claude");
    fs::create_dir_all(&claude_dir).unwrap();

    // Should not find config (file doesn't exist)
    let found = find_project_config(Some(&project));

    assert!(found.is_none());
}
