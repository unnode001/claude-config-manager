//! Path resolution and project detection
//!
//! This module provides functionality for:
//! - Resolving platform-specific configuration paths
//! - Detecting project configuration files by searching upward

use std::path::{Path, PathBuf};

/// Get the default global configuration directory
///
/// Returns platform-specific path:
/// - Windows: `%APPDATA%\claude`
/// - macOS: `~/Library/Application Support/Claude`
/// - Linux: `~/.config/claude`
pub fn get_global_config_dir() -> PathBuf {
    // Use dirs crate for cross-platform path resolution
    if cfg!(windows) {
        // Windows: %APPDATA%\claude
        dirs::config_dir()
            .map(|dir| dir.join("claude"))
            .unwrap_or_else(|| {
                // Fallback if dirs crate fails
                let mut path = PathBuf::new();
                if let Ok(appdata) = std::env::var("APPDATA") {
                    path.push(appdata);
                }
                path.push("claude");
                path
            })
    } else if cfg!(target_os = "macos") {
        // macOS: ~/Library/Application Support/Claude
        dirs::config_dir()
            .map(|dir| dir.join("Claude"))
            .unwrap_or_else(|| {
                // Fallback
                let mut home = PathBuf::from("~");
                home.push("Library");
                home.push("Application Support");
                home.push("Claude");
                home
            })
    } else {
        // Linux/Unix: ~/.config/claude
        dirs::config_dir()
            .map(|dir| dir.join("claude"))
            .unwrap_or_else(|| {
                // Fallback
                let mut home = PathBuf::from("~");
                home.push(".config");
                home.push("claude");
                home
            })
    }
}

/// Get the global configuration file path
///
/// Returns `<config_dir>/config.json`
pub fn get_global_config_path() -> PathBuf {
    get_global_config_dir().join("config.json")
}

/// Find project configuration by searching upward
///
/// Starts from `start_dir` and searches upward for `.claude/config.json`.
/// Stops at filesystem root or Git repository root.
///
/// # Arguments
/// * `start_dir` - Directory to start searching from
///
/// # Returns
/// - `Some(path)` if project config found
/// - `None` if not found
///
/// # Examples
/// ```
/// use claude_config_manager_core::find_project_config;
///
/// // Start from current directory
/// let project_config = find_project_config(std::env::current_dir().ok().as_deref());
/// ```
pub fn find_project_config(start_dir: Option<&Path>) -> Option<PathBuf> {
    // Convert start_dir to PathBuf, or use current directory
    let mut current: PathBuf = match start_dir {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir().ok()?,
    };

    loop {
        // Check if .claude/config.json exists in current directory
        let config_path = current.join(".claude").join("config.json");
        if config_path.exists() {
            return Some(config_path);
        }

        // Check if we've hit a Git repository root (stop searching)
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return None;
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) if parent != current => {
                current = parent.to_path_buf();
            }
            _ => {
                // Reached filesystem root
                return None;
            }
        }
    }
}

/// Expand tilde (~) in path to home directory
///
/// # Arguments
/// * `path` - Path that may start with ~
///
/// # Returns
/// Expanded path with ~ replaced by home directory
pub fn expand_tilde(path: &Path) -> PathBuf {
    // Convert to string for processing
    let path_str = path.to_string_lossy();

    if path_str.starts_with('~') {
        // Replace ~ with home directory
        if let Some(home) = dirs::home_dir() {
            let rest = &path_str[1..]; // Skip ~
            return home.join(rest.trim_start_matches('/'));
        }
    }

    path.to_path_buf()
}

/// Get the backup directory path
///
/// Returns `<config_dir>/backups`
pub fn get_backup_dir() -> PathBuf {
    get_global_config_dir().join("backups")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // TDD Test 1: Global config dir returns valid path
    #[test]
    fn test_get_global_config_dir_returns_valid_path() {
        let config_dir = get_global_config_dir();

        // Should not be empty
        assert!(!config_dir.as_os_str().is_empty());

        // Last component should be "claude" or "Claude"
        let last_component = config_dir.file_name();
        assert!(last_component.is_some());
    }

    // TDD Test 2: Global config path has config.json
    #[test]
    fn test_get_global_config_path_ends_with_config_json() {
        let config_path = get_global_config_path();

        // Should end with config.json
        assert!(config_path.to_string_lossy().ends_with("config.json"));
    }

    // TDD Test 3: Find project config in nested directory
    #[test]
    fn test_find_project_config_in_nested_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("nested").join("project");

        // Create directory structure
        fs::create_dir_all(&project_dir).unwrap();
        fs::create_dir_all(project_dir.join(".claude")).unwrap();

        // Create config file
        let config_path = project_dir.join(".claude").join("config.json");
        fs::write(&config_path, "{}").unwrap();

        // Start from nested directory
        let found = find_project_config(Some(&project_dir));

        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    // TDD Test 4: No project config returns None
    #[test]
    fn test_find_project_config_returns_none_when_missing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("no-config");

        fs::create_dir_all(&project_dir).unwrap();

        let found = find_project_config(Some(&project_dir));

        assert!(found.is_none());
    }

    // TDD Test 5: Stops at Git repository root
    #[test]
    fn test_stops_at_git_repository_root() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let git_root = temp_dir.path().join("git-repo");
        let nested = git_root.join("nested");

        // Create Git repository root
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir_all(git_root.join(".git")).unwrap();

        // Create config ABOVE Git root (should not be found)
        let config_above = temp_dir.path().join(".claude").join("config.json");
        fs::create_dir_all(config_above.parent().unwrap()).unwrap();
        fs::write(&config_above, "{}").unwrap();

        let found = find_project_config(Some(&nested));

        // Should not find config above Git root
        assert!(found.is_none());
    }

    // TDD Test 6: Expand tilde for Unix systems
    #[test]
    fn test_expand_tilde_replaces_tilde() {
        let path = PathBuf::from("~/projects");
        let expanded = expand_tilde(&path);

        // Should not start with ~ after expansion
        let path_str = expanded.to_string_lossy();
        assert!(!path_str.starts_with('~'));
    }

    // TDD Test 7: Expand regular path unchanged
    #[test]
    fn test_expand_regular_path_unchanged() {
        let path = PathBuf::from("/usr/local/bin");
        let expanded = expand_tilde(&path);

        assert_eq!(expanded, path);
    }

    // TDD Test 8: Empty path handles gracefully
    #[test]
    fn test_expand_empty_path() {
        let path = PathBuf::from("");
        let expanded = expand_tilde(&path);

        assert_eq!(expanded, path);
    }
}
