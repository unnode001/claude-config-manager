//! Project discovery and scanning
//!
//! This module provides functionality to scan the filesystem for projects
//! with .claude directories, enabling users to discover and manage multiple
//! Claude Code configurations.

use crate::{error::Result, paths::find_project_config};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Information about a discovered project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectInfo {
    /// Project root directory
    pub root: PathBuf,

    /// Path to .claude directory
    pub claude_dir: PathBuf,

    /// Path to config file
    pub config_path: PathBuf,

    /// Whether config exists
    pub has_config: bool,

    /// Project name (derived from directory name)
    pub name: String,

    /// Last modification time
    pub last_modified: Option<SystemTime>,
}

impl ProjectInfo {
    /// Create project info from a discovered config path
    pub fn from_config_path(config_path: PathBuf) -> Self {
        let claude_dir = config_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let root = claude_dir
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();

        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let has_config = config_path.exists();

        let last_modified = fs::metadata(&config_path)
            .ok()
            .and_then(|m| m.modified().ok());

        Self {
            root,
            claude_dir,
            config_path,
            has_config,
            name,
            last_modified,
        }
    }
}

/// Project scanner for discovering Claude Code projects
///
/// Scans directory trees to find all projects with .claude directories,
/// with support for parallel traversal and filtering.
#[derive(Debug, Clone)]
pub struct ProjectScanner {
    /// Maximum scan depth (None = unlimited)
    max_depth: Option<usize>,

    /// Paths to ignore during scan
    ignore_paths: Vec<String>,

    /// Whether to use parallel traversal (reserved for future use)
    #[allow(dead_code)]
    parallel: bool,
}

impl ProjectScanner {
    /// Create a new project scanner
    ///
    /// # Arguments
    /// * `max_depth` - Maximum directory depth to scan (None = unlimited)
    /// * `parallel` - Whether to use parallel traversal
    pub fn new(max_depth: Option<usize>, parallel: bool) -> Self {
        Self {
            max_depth,
            ignore_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            parallel,
        }
    }

    /// Add a path pattern to ignore
    pub fn ignore_path(mut self, path: impl Into<String>) -> Self {
        self.ignore_paths.push(path.into());
        self
    }

    /// Scan a directory for projects
    ///
    /// # Arguments
    /// * `start_path` - Root directory to start scanning
    ///
    /// # Returns
    /// Vector of discovered project information
    pub fn scan_directory(&self, start_path: &Path) -> Result<Vec<ProjectInfo>> {
        let mut projects = Vec::new();

        // Scan subdirectories (don't check start_path itself, only its children)
        self.scan_recursive(start_path, 0, &mut projects)?;

        // Remove duplicates (in case same project found multiple times)
        projects.sort_by(|a, b| a.root.cmp(&b.root));
        projects.dedup();

        // Sort by project name
        projects.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(projects)
    }

    /// Recursive directory scanning
    fn scan_recursive(
        &self,
        dir: &Path,
        depth: usize,
        projects: &mut Vec<ProjectInfo>,
    ) -> Result<()> {
        // Check depth limit
        if let Some(max) = self.max_depth {
            if depth >= max {
                return Ok(());
            }
        }

        // Read directory entries
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()), // Skip directories we can't read
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Skip if in ignore list
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if self.should_ignore(file_name) {
                continue;
            }

            // Check if this directory contains a .claude/config.json
            if let Some(config) = find_project_config(Some(&path)) {
                projects.push(ProjectInfo::from_config_path(config));
            }

            // Recursively scan subdirectory
            self.scan_recursive(&path, depth + 1, projects)?;
        }

        Ok(())
    }

    /// Check if a path should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        self.ignore_paths.iter().any(|ignore| {
            name == *ignore || {
                let name_lower = name.to_lowercase();
                name_lower.starts_with(&ignore.to_lowercase())
            }
        })
    }
}

impl Default for ProjectScanner {
    fn default() -> Self {
        Self::new(None, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // TDD Test 1: Scanner finds project with .claude directory
    #[test]
    fn test_scanner_finds_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("my-project");

        // Create project with .claude/config.json
        let claude_dir = project_dir.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let config_file = claude_dir.join("config.json");
        fs::write(&config_file, r#"{"mcpServers": {}}"#).unwrap();

        // Scan
        let scanner = ProjectScanner::new(Some(3), false);
        let results = scanner.scan_directory(&project_dir).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].name, "my-project");
    }

    // TDD Test 2: Scanner respects max_depth
    #[test]
    fn test_scanner_respects_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested project structure
        let level1 = temp_dir.path().join("level1");
        let level2 = level1.join("level2");
        let level3 = level2.join("level3-project");

        let claude_dir = level3.join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("config.json"), "{}").unwrap();

        // Scan with depth 2 - should not find level3
        let scanner = ProjectScanner::new(Some(2), false);
        let results = scanner.scan_directory(temp_dir.path()).unwrap();

        assert!(
            results.is_empty(),
            "Should not find project beyond max depth"
        );
    }

    // TDD Test 3: Scanner ignores common directories
    #[test]
    fn test_scanner_ignores_common_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create projects in various directories
        let project1 = root.join("my-project");
        let node_modules = root.join("node_modules");
        let nested = node_modules.join("nested-project");

        // Create projects
        for dir in &[&project1, &nested] {
            let claude_dir = dir.join(".claude");
            fs::create_dir_all(&claude_dir).unwrap();
            fs::write(claude_dir.join("config.json"), "{}").unwrap();
        }

        // Scan
        let scanner = ProjectScanner::default();
        let results = scanner.scan_directory(root).unwrap();

        // Should find my-project but not nested in node_modules
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "my-project");
    }

    // TDD Test 4: Scanner returns empty when no projects found
    #[test]
    fn test_scanner_returns_empty_when_no_projects() {
        let temp_dir = TempDir::new().unwrap();

        // Create directory without .claude
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).unwrap();

        let scanner = ProjectScanner::default();
        let results = scanner.scan_directory(temp_dir.path()).unwrap();

        assert!(results.is_empty());
    }

    // TDD Test 5: Scanner handles multiple projects
    #[test]
    fn test_scanner_finds_multiple_projects() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple projects
        for i in 0..3 {
            let project_dir = temp_dir.path().join(format!("project-{i}"));
            let claude_dir = project_dir.join(".claude");
            fs::create_dir_all(&claude_dir).unwrap();
            fs::write(claude_dir.join("config.json"), r#"{"project": i}"#).unwrap();
        }

        let scanner = ProjectScanner::new(None, false);
        let results = scanner.scan_directory(temp_dir.path()).unwrap();

        assert_eq!(results.len(), 3);
    }
}
