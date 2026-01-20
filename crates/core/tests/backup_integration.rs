//! Integration tests for backup system with real filesystem
//!
//! These tests verify backup behavior in realistic scenarios.

use claude_config_manager_core::{BackupManager, ConfigError};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_backup_workflow_full_cycle() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("claude");
    let backup_dir = temp_dir.path().join("backups");
    let config_file = config_dir.join("config.json");

    fs::create_dir_all(&config_dir).unwrap();

    // Create initial config
    let mut file = File::create(&config_file).unwrap();
    file.write_all(b"{\"version\": 1}").unwrap();

    // Create backup manager
    let manager = BackupManager::new(&backup_dir, None);

    // Create backup
    let backup_path = manager.create_backup(&config_file).unwrap();
    assert!(backup_path.exists());

    // Modify original file
    let mut file = File::create(&config_file).unwrap();
    file.write_all(b"{\"version\": 2}").unwrap();

    // Create another backup
    let backup_path2 = manager.create_backup(&config_file).unwrap();
    assert!(backup_path2.exists());

    // List backups
    let backups = manager.list_backups(&config_file).unwrap();
    assert_eq!(backups.len(), 2);

    // Verify backup content matches original at time of backup
    let backup_content = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(backup_content, "{\"version\": 1}");
}

#[test]
fn test_backup_preserves_original_file() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    // Create original file
    let original_content = b"{\"test\": \"data\"}";
    let mut file = File::create(&config_file).unwrap();
    file.write_all(original_content).unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Create backup
    manager.create_backup(&config_file).unwrap();

    // Verify original file unchanged
    let current_content = fs::read_to_string(&config_file).unwrap();
    assert_eq!(current_content.as_bytes(), original_content);
}

#[test]
fn test_multiple_backups_different_files() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let manager = BackupManager::new(&backup_dir, None);

    // Create multiple config files
    let config1 = temp_dir.path().join("config1.json");
    let config2 = temp_dir.path().join("config2.json");

    File::create(&config1).unwrap().write_all(b"{\"id\": 1}").unwrap();
    File::create(&config2).unwrap().write_all(b"{\"id\": 2}").unwrap();

    // Create backups for both files
    manager.create_backup(&config1).unwrap();
    manager.create_backup(&config2).unwrap();

    // List backups for each file
    let backups1 = manager.list_backups(&config1).unwrap();
    let backups2 = manager.list_backups(&config2).unwrap();

    assert_eq!(backups1.len(), 1);
    assert_eq!(backups2.len(), 1);

    // Verify backups are distinct
    assert_ne!(backups1[0].path, backups2[0].path);
}

#[test]
fn test_backup_directory_created_automatically() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("nonexistent").join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Backup directory should be created automatically
    let backup_path = manager.create_backup(&config_file).unwrap();
    assert!(backup_dir.exists());
    assert!(backup_path.exists());
}

#[test]
fn test_backup_with_unicode_filename() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("配置文件.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Should handle unicode filenames
    let result = manager.create_backup(&config_file);
    assert!(result.is_ok());

    let backup_path = result.unwrap();
    assert!(backup_path.exists());
}

#[test]
fn test_large_file_backup() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("large_config.json");

    // Create a large config file (1MB)
    let large_content = "{\"data\": \"".to_string() + &"x".repeat(1_000_000) + "\"}";
    File::create(&config_file)
        .unwrap()
        .write_all(large_content.as_bytes()).unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Should handle large files
    let backup_path = manager.create_backup(&config_file).unwrap();
    assert!(backup_path.exists());

    // Verify file size
    let backup_size = fs::metadata(&backup_path).unwrap().len();
    assert!(backup_size > 1_000_000);
}

#[test]
fn test_cleanup_removes_correct_backups() {
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, Some(3));

    // Create 5 backups
    let mut backup_paths = Vec::new();
    for _ in 0..5 {
        let path = manager.create_backup(&config_file).unwrap();
        backup_paths.push(path.clone());
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Cleanup
    let removed = manager.cleanup_old_backups(&config_file).unwrap();
    assert_eq!(removed, 2);

    // Verify only 3 backups remain
    let backups = manager.list_backups(&config_file).unwrap();
    assert_eq!(backups.len(), 3);

    // Verify remaining backups exist
    for backup in &backups {
        assert!(
            PathBuf::from(&backup.path).exists(),
            "Remaining backup should exist: {}",
            backup.path
        );
    }

    // Verify that exactly 2 backups were removed from filesystem
    let remaining_count = backup_paths.iter().filter(|p| p.exists()).count();
    assert_eq!(remaining_count, 3, "Should have exactly 3 remaining backups");
}

#[test]
fn test_error_handling_on_permission_denied() {
    // This test verifies proper error messages when permissions are denied
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Create a backup
    manager.create_backup(&config_file).unwrap();

    // Make backup directory read-only (platform-specific)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&backup_dir).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&backup_dir, perms).unwrap();

        // Try to create another backup - should fail with helpful error
        let result = manager.create_backup(&config_file);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Permission denied") || message.contains("filesystem"));

        // Restore permissions for cleanup
        perms.set_readonly(false);
        fs::set_permissions(&backup_dir, perms).unwrap();
    }
}

#[test]
fn test_concurrent_backup_safety() {
    // Test that multiple backups of the same file work correctly
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Create multiple backups rapidly
    for _ in 0..3 {
        let result = manager.create_backup(&config_file);
        assert!(result.is_ok(), "Each backup should succeed");
    }

    // Verify all 3 backups exist
    let backups = manager.list_backups(&config_file).unwrap();
    assert_eq!(backups.len(), 3);
}

// T104: Backup/restore workflow integration tests
#[test]
fn test_restore_workflow_full_cycle() {
    // TDD Test: Full backup and restore workflow
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    // Create initial config
    let original_content = b"{\"version\": 1, \"data\": \"original\"}";
    File::create(&config_file).unwrap().write_all(original_content).unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Create backup
    let backup_path = manager.create_backup(&config_file).unwrap();
    assert!(backup_path.exists());

    // Modify the original file
    let modified_content = b"{\"version\": 2, \"data\": \"modified\"}";
    File::create(&config_file).unwrap().write_all(modified_content).unwrap();

    // Verify file was modified
    let current = fs::read_to_string(&config_file).unwrap();
    assert_eq!(current, String::from_utf8_lossy(modified_content));

    // Restore from backup
    let restored_path = manager.restore_backup(&backup_path).unwrap();
    assert_eq!(restored_path, config_file);

    // Verify content was restored
    let restored = fs::read_to_string(&config_file).unwrap();
    assert_eq!(restored, String::from_utf8_lossy(original_content));
}

#[test]
fn test_restore_selective_backup() {
    // TDD Test: Restore specific backup from multiple backups
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    let manager = BackupManager::new(&backup_dir, None);

    // Create first version and backup
    let v1 = b"{\"version\": 1}";
    File::create(&config_file).unwrap().write_all(v1).unwrap();
    let backup1 = manager.create_backup(&config_file).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create second version and backup
    let v2 = b"{\"version\": 2}";
    File::create(&config_file).unwrap().write_all(v2).unwrap();
    let backup2 = manager.create_backup(&config_file).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Create third version and backup
    let v3 = b"{\"version\": 3}";
    File::create(&config_file).unwrap().write_all(v3).unwrap();
    let backup3 = manager.create_backup(&config_file).unwrap();

    // Restore to first version
    manager.restore_backup(&backup1).unwrap();
    let content = fs::read_to_string(&config_file).unwrap();
    assert_eq!(content, String::from_utf8_lossy(v1));

    // Restore to third version
    manager.restore_backup(&backup3).unwrap();
    let content = fs::read_to_string(&config_file).unwrap();
    assert_eq!(content, String::from_utf8_lossy(v3));
}

#[test]
fn test_restore_creates_missing_parent_directory() {
    // TDD Test: Restore creates parent directory if missing
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    // Create config file and backup
    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);
    let backup_path = manager.create_backup(&config_file).unwrap();

    // Remove the config file
    fs::remove_file(&config_file).unwrap();
    assert!(!config_file.exists());

    // Restore should recreate the file
    let restored_path = manager.restore_backup(&backup_path).unwrap();
    assert!(restored_path.exists());
    assert_eq!(restored_path, config_file);
}

#[test]
fn test_restore_nonexistent_backup_fails() {
    // TDD Test: Restore of non-existent backup fails with clear error
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");

    let manager = BackupManager::new(&backup_dir, None);

    let fake_backup = backup_dir.join("config_20250120_120000.000.json");
    let result = manager.restore_backup(&fake_backup);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let message = err.to_string();

    // Error should mention "not found"
    assert!(message.contains("not found") || message.contains("No such file"));
}

#[test]
fn test_backup_list_order() {
    // TDD Test: Backup list is ordered newest first
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, None);

    // Create multiple backups
    let mut backup_paths = Vec::new();
    for i in 0..3 {
        File::create(&config_file).unwrap().write_all(format!("{{\"v\":{}}}", i).as_bytes()).unwrap();
        let path = manager.create_backup(&config_file).unwrap();
        backup_paths.push(path);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // List backups
    let backups = manager.list_backups(&config_file).unwrap();
    assert_eq!(backups.len(), 3);

    // Verify ordering: newest first
    // The last created backup should be first in the list
    assert!(backups[0].created_at >= backups[1].created_at);
    assert!(backups[1].created_at >= backups[2].created_at);
}

#[test]
fn test_backup_restore_with_cleanup() {
    // TDD Test: Backup cleanup doesn't affect recent backups used for restore
    let temp_dir = TempDir::new().unwrap();
    let backup_dir = temp_dir.path().join("backups");
    let config_file = temp_dir.path().join("config.json");

    File::create(&config_file).unwrap().write_all(b"{}").unwrap();

    let manager = BackupManager::new(&backup_dir, Some(2)); // Keep only 2

    // Create 5 backups
    for i in 0..5 {
        File::create(&config_file).unwrap().write_all(format!("{{\"v\":{}}}", i).as_bytes()).unwrap();
        manager.create_backup(&config_file).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Cleanup should remove 3 oldest backups
    let removed = manager.cleanup_old_backups(&config_file).unwrap();
    assert_eq!(removed, 3);

    // Get remaining backups
    let backups = manager.list_backups(&config_file).unwrap();
    assert_eq!(backups.len(), 2);

    // Restore from most recent backup should still work
    let most_recent_backup = PathBuf::from(&backups[0].path);
    let result = manager.restore_backup(&most_recent_backup);
    assert!(result.is_ok(), "Restore from recent backup should work after cleanup");
}
