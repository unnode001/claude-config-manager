//! Backup system for configuration files
//!
//! This module provides functionality to create, list, and manage backups
//! of configuration files to ensure data safety.

use crate::{error::{ConfigError, Result}, types::BackupInfo};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

/// Default number of backups to retain
const DEFAULT_RETENTION_COUNT: usize = 10;

/// Backup manager for configuration files
///
/// Manages backup creation, listing, and cleanup with retention policies.
#[derive(Debug, Clone)]
pub struct BackupManager {
    /// Backup directory path
    backup_dir: PathBuf,
    /// Number of backups to retain
    retention_count: usize,
}

impl BackupManager {
    /// Create a new BackupManager
    ///
    /// # Arguments
    /// * `backup_dir` - Directory to store backups
    /// * `retention_count` - Number of backups to retain (default: 10)
    pub fn new(backup_dir: impl Into<PathBuf>, retention_count: Option<usize>) -> Self {
        Self {
            backup_dir: backup_dir.into(),
            retention_count: retention_count.unwrap_or(DEFAULT_RETENTION_COUNT),
        }
    }

    /// Create a backup of the specified file
    ///
    /// # Arguments
    /// * `file_path` - Path to the file to backup
    ///
    /// # Returns
    /// Path to the created backup file
    ///
    /// # Errors
    /// Returns an error if:
    /// - The source file doesn't exist
    /// - Backup directory cannot be created
    /// - File cannot be copied
    pub fn create_backup(&self, file_path: &Path) -> Result<PathBuf> {
        // Verify source file exists
        if !file_path.exists() {
            return Err(ConfigError::not_found(file_path));
        }

        // Create backup directory if it doesn't exist
        if !self.backup_dir.exists() {
            fs::create_dir_all(&self.backup_dir).map_err(|e| {
                ConfigError::filesystem(
                    "create backup directory",
                    &self.backup_dir,
                    e,
                )
            })?;
        }

        // Generate backup filename with timestamp
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.3f");
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("config");
        let extension = file_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("json");

        let backup_name = format!("{}_{}.{}", file_stem, timestamp, extension);
        let backup_path = self.backup_dir.join(&backup_name);

        // Copy file to backup location
        fs::copy(file_path, &backup_path).map_err(|e| {
            ConfigError::filesystem("copy file to backup", file_path, e)
        })?;

        tracing::debug!(
            "Created backup: {} -> {}",
            file_path.display(),
            backup_path.display()
        );

        Ok(backup_path)
    }

    /// List all available backups for a specific file
    ///
    /// # Arguments
    /// * `original_file` - Path to the original file
    ///
    /// # Returns
    /// Vector of backup information, sorted by creation time (newest first)
    pub fn list_backups(&self, original_file: &Path) -> Result<Vec<BackupInfo>> {
        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        let file_stem = original_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("config");

        let mut backups = Vec::new();

        for entry in fs::read_dir(&self.backup_dir).map_err(|e| {
            ConfigError::filesystem("read backup directory", &self.backup_dir, e)
        })? {
            let entry = entry.map_err(|e| {
                ConfigError::filesystem("read backup entry", &self.backup_dir, e)
            })?;

            let path = entry.path();

            // Check if filename matches pattern: <file_stem>_<timestamp>.<ext>
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&format!("{}_", file_stem)) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let created_at: DateTime<Utc> = modified.into();
                            let size = metadata.len();

                            backups.push(BackupInfo {
                                path: path.to_string_lossy().to_string(),
                                original_path: original_file.to_string_lossy().to_string(),
                                created_at,
                                size,
                            });
                        }
                    }
                }
            }
        }

        // Sort by creation time, newest first
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Clean up old backups according to retention policy
    ///
    /// Removes oldest backups beyond the retention count.
    ///
    /// # Arguments
    /// * `original_file` - Path to the original file
    ///
    /// # Returns
    /// Number of backups removed
    pub fn cleanup_old_backups(&self, original_file: &Path) -> Result<usize> {
        let mut backups = self.list_backups(original_file)?;

        // Keep only the most recent N backups
        if backups.len() <= self.retention_count {
            return Ok(0);
        }

        let backups_to_remove = backups.split_off(self.retention_count);
        let mut removed_count = 0;

        for backup in backups_to_remove {
            fs::remove_file(&backup.path).map_err(|e| {
                ConfigError::filesystem("remove old backup", Path::new(&backup.path), e)
            })?;

            tracing::debug!("Removed old backup: {}", backup.path);
            removed_count += 1;
        }

        Ok(removed_count)
    }

    /// Get the backup directory path
    pub fn backup_dir(&self) -> &Path {
        &self.backup_dir
    }

    /// Get the retention count
    pub fn retention_count(&self) -> usize {
        self.retention_count
    }

    /// Restore a backup to the original file location
    ///
    /// # Arguments
    /// * `backup_path` - Path to the backup file to restore
    ///
    /// # Returns
    /// Path to the restored file (original location)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The backup file doesn't exist
    /// - The original file's parent directory doesn't exist
    /// - File cannot be copied
    pub fn restore_backup(&self, backup_path: &Path) -> Result<PathBuf> {
        // Verify backup file exists
        if !backup_path.exists() {
            return Err(ConfigError::not_found(backup_path));
        }

        // Extract original file path from backup name
        // Backup format: <file_stem>_<timestamp>.<ext>
        let file_name = backup_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                ConfigError::validation_failed(
                    "BackupRestore",
                    format!("Invalid backup file name: {:?}", backup_path.file_name()),
                    "Ensure the backup file follows the naming pattern: <filename>_<timestamp>.<ext>",
                )
            })?;

        // Parse the backup filename to get the original file stem
        // Format: config_20250120_123456.789.json
        if let Some(stem_with_timestamp) = backup_path.file_stem().and_then(|s| s.to_str()) {
            if let Some(original_stem) = stem_with_timestamp.split('_').next() {
                let extension = backup_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("json");

                // Build the original file path (in parent directory of backups)
                let original_file = self.backup_dir
                    .parent()
                    .unwrap_or(&self.backup_dir)
                    .join(format!("{}.{}", original_stem, extension));

                // Ensure parent directory exists
                if let Some(parent) = original_file.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| {
                            ConfigError::filesystem("create parent directory", parent, e)
                        })?;
                    }
                }

                // Copy backup to original location
                fs::copy(backup_path, &original_file).map_err(|e| {
                    ConfigError::filesystem("restore backup", &original_file, e)
                })?;

                tracing::info!(
                    "Restored backup: {} -> {}",
                    backup_path.display(),
                    original_file.display()
                );

                return Ok(original_file);
            }
        }

        Err(ConfigError::validation_failed(
            "BackupRestore",
            format!("Could not determine original file path from backup name: {}", file_name),
            "Ensure the backup file follows the naming pattern: <filename>_<timestamp>.<ext>",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    // TDD Test 1: Create backup successfully
    #[test]
    fn test_create_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        // Create a test file
        let test_file = temp_dir.path().join("config.json");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"{\"test\": \"data\"}").unwrap();

        // Create backup
        let backup_path = manager.create_backup(&test_file).unwrap();

        // Verify backup exists
        assert!(backup_path.exists());
        assert!(backup_path.starts_with(&backup_dir));

        // Verify backup content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        let original_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(backup_content, original_content);
    }

    // TDD Test 2: Create backup fails if source doesn't exist
    #[test]
    fn test_create_backup_nonexistent_source() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        let nonexistent_file = temp_dir.path().join("nonexistent.json");
        let result = manager.create_backup(&nonexistent_file);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // TDD Test 3: List backups returns empty when none exist
    #[test]
    fn test_list_backups_empty() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        let original_file = temp_dir.path().join("config.json");
        let backups = manager.list_backups(&original_file).unwrap();

        assert!(backups.is_empty());
    }

    // TDD Test 4: List backups returns all backups
    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        // Create a test file
        let test_file = temp_dir.path().join("config.json");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"{\"test\": \"data\"}").unwrap();

        // Create multiple backups with longer delay to ensure different timestamps
        manager.create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.create_backup(&test_file).unwrap();

        // List backups
        let backups = manager.list_backups(&test_file).unwrap();

        assert_eq!(backups.len(), 2);

        // Verify sorted by creation time (newest first)
        // Note: Some file systems have limited timestamp precision,
        // so we just verify we have 2 backups and the list is sorted
        assert!(backups.len() == 2);

        // Verify that if timestamps differ, the order is correct
        if backups[0].created_at != backups[1].created_at {
            assert!(backups[0].created_at > backups[1].created_at);
        }
    }

    // TDD Test 5: Cleanup old backups removes excess backups
    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, Some(2)); // Keep only 2

        // Create a test file
        let test_file = temp_dir.path().join("config.json");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"{\"test\": \"data\"}").unwrap();

        // Create 5 backups
        for _ in 0..5 {
            manager.create_backup(&test_file).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Cleanup should remove 3 oldest backups
        let removed = manager.cleanup_old_backups(&test_file).unwrap();
        assert_eq!(removed, 3);

        // Verify only 2 backups remain
        let backups = manager.list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 2);
    }

    // TDD Test 6: Cleanup doesn't remove backups under retention limit
    #[test]
    fn test_cleanup_preserves_retained_backups() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, Some(5));

        // Create a test file
        let test_file = temp_dir.path().join("config.json");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"{\"test\": \"data\"}").unwrap();

        // Create 3 backups
        for _ in 0..3 {
            manager.create_backup(&test_file).unwrap();
        }

        // Cleanup should not remove any backups
        let removed = manager.cleanup_old_backups(&test_file).unwrap();
        assert_eq!(removed, 0);

        // Verify all 3 backups remain
        let backups = manager.list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 3);
    }

    // TDD Test 7: Backup manager properties
    #[test]
    fn test_backup_manager_properties() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let retention = 15;
        let manager = BackupManager::new(&backup_dir, Some(retention));

        assert_eq!(manager.backup_dir(), &backup_dir);
        assert_eq!(manager.retention_count(), retention);
    }

    // TDD Test 8: BackupInfo contains correct metadata
    #[test]
    fn test_backup_info_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        // Create a test file with known content
        let test_file = temp_dir.path().join("config.json");
        let content = b"{\"test\": \"data\"}";
        let mut file = File::create(&test_file).unwrap();
        file.write_all(content).unwrap();

        // Create backup
        manager.create_backup(&test_file).unwrap();

        // List backups
        let backups = manager.list_backups(&test_file).unwrap();
        assert_eq!(backups.len(), 1);

        let backup = &backups[0];
        assert_eq!(backup.original_path, test_file.to_string_lossy().to_string());
        assert!(backup.size > 0);
        assert!(backup.path.contains("config_"));
    }

    // TDD Test 9: Restore backup successfully
    #[test]
    fn test_restore_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        // Create a test file with original content
        let test_file = temp_dir.path().join("config.json");
        let original_content = b"{\"test\": \"original\"}";
        let mut file = File::create(&test_file).unwrap();
        file.write_all(original_content).unwrap();

        // Create backup
        let backup_path = manager.create_backup(&test_file).unwrap();

        // Modify the original file
        let modified_content = b"{\"test\": \"modified\"}";
        let mut file = File::create(&test_file).unwrap();
        file.write_all(modified_content).unwrap();

        // Restore from backup
        let restored_path = manager.restore_backup(&backup_path).unwrap();

        // Verify the restored content matches the backup
        let restored_content = fs::read_to_string(&restored_path).unwrap();
        assert_eq!(restored_content, String::from_utf8_lossy(original_content));
    }

    // TDD Test 10: Restore non-existent backup fails
    #[test]
    fn test_restore_nonexistent_backup_fails() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        let nonexistent_backup = backup_dir.join("config_20250120_120000.000.json");
        let result = manager.restore_backup(&nonexistent_backup);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // TDD Test 11: Restore multiple backups
    #[test]
    fn test_restore_specific_backup() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        let manager = BackupManager::new(&backup_dir, None);

        // Create a test file
        let test_file = temp_dir.path().join("config.json");

        // Create first backup
        let content1 = b"{\"version\": 1}";
        let mut file = File::create(&test_file).unwrap();
        file.write_all(content1).unwrap();
        let backup1 = manager.create_backup(&test_file).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Create second backup
        let content2 = b"{\"version\": 2}";
        let mut file = File::create(&test_file).unwrap();
        file.write_all(content2).unwrap();
        let backup2 = manager.create_backup(&test_file).unwrap();

        // Restore first backup
        let restored_path = manager.restore_backup(&backup1).unwrap();
        let restored_content = fs::read_to_string(&restored_path).unwrap();
        assert_eq!(restored_content, String::from_utf8_lossy(content1));

        // Restore second backup
        let restored_path = manager.restore_backup(&backup2).unwrap();
        let restored_content = fs::read_to_string(&restored_path).unwrap();
        assert_eq!(restored_content, String::from_utf8_lossy(content2));
    }
}
