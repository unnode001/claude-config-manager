//! Performance Benchmarks
//!
//! Tests the performance of critical operations to ensure they meet targets:
//! - Config parsing: < 10ms
//! - Config writing: < 50ms
//! - Config merging: < 5ms

use std::fs;
use std::time::Instant;
use tempfile::TempDir;
use claude_config_manager_core::{ConfigManager, ClaudeConfig, merge_configs};

/// Benchmark config parsing performance
#[test]
fn bench_config_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    // Create a typical config file
    let config_content = r#"{
        "customInstructions": ["You are a helpful assistant."],
        "mcpServers": {
            "npx": {
                "command": "npx",
                "args": ["-y"],
                "enabled": true
            },
            "uvx": {
                "command": "uvx",
                "enabled": false
            }
        }
    }"#;

    fs::write(&config_path, config_content).unwrap();

    // Benchmark parsing
    let start = Instant::now();
    let manager = ConfigManager::new(temp_dir.path().join("backups"));
    let _config = manager.read_config(&config_path).unwrap();
    let duration = start.elapsed();

    println!("Config parsing took: {:?}", duration);

    // Assert parsing is under 10ms
    assert!(duration.as_millis() < 10, "Config parsing took {:?}, expected < 10ms", duration);
}

/// Benchmark config writing performance
#[test]
fn bench_config_writing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut config = ClaudeConfig::new();
    config.custom_instructions = Some(vec!["Test instructions".to_string()]);

    // Benchmark writing
    let start = Instant::now();
    let manager = ConfigManager::new(temp_dir.path().join("backups"));
    manager.write_config_with_backup(&config_path, &config).unwrap();
    let duration = start.elapsed();

    println!("Config writing took: {:?}", duration);

    // Assert writing is under 50ms
    assert!(duration.as_millis() < 50, "Config writing took {:?}, expected < 50ms", duration);
}

/// Benchmark config merging performance
#[test]
fn bench_config_merging() {
    let mut global = ClaudeConfig::new();
    global.custom_instructions = Some(vec!["Global instructions".to_string()]);

    let mut project = ClaudeConfig::new();
    project.custom_instructions = Some(vec!["Project instructions".to_string()]);

    // Benchmark merging
    let start = Instant::now();
    let _merged = merge_configs(&global, &project);
    let duration = start.elapsed();

    println!("Config merging took: {:?}", duration);

    // Assert merging is under 5ms
    assert!(duration.as_millis() < 5, "Config merging took {:?}, expected < 5ms", duration);
}

/// Benchmark large config parsing (stress test)
#[test]
fn bench_large_config_parsing() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("large_config.json");

    // Create a large config with many MCP servers
    let mut config = ClaudeConfig::new();
    let mut mcp_servers = std::collections::HashMap::new();

    for i in 0..100 {
        let server = claude_config_manager_core::McpServer::new(
            &format!("server-{}", i),
            "npx",
            vec!["-y".to_string()]
        );
        mcp_servers.insert(format!("server-{}", i), server);
    }

    config.mcp_servers = Some(mcp_servers);

    // Write to file
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, json).unwrap();

    // Benchmark parsing large config
    let start = Instant::now();
    let manager = ConfigManager::new(temp_dir.path().join("backups"));
    let _parsed_config = manager.read_config(&config_path).unwrap();
    let duration = start.elapsed();

    println!("Large config parsing (100 servers) took: {:?}", duration);

    // For large configs, we allow more time but it should still be reasonable
    assert!(duration.as_millis() < 50, "Large config parsing took {:?}, expected < 50ms", duration);
}

/// Benchmark repeated operations (stability test)
#[test]
fn bench_repeated_parse_write_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    let backup_dir = temp_dir.path().join("backups");

    let mut config = ClaudeConfig::new();
    config.custom_instructions = Some(vec!["Test instructions".to_string()]);
    config.mcp_servers = Some(std::collections::HashMap::new());

    let manager = ConfigManager::new(&backup_dir);

    // Run 10 parse-write cycles
    let iterations = 10;
    let start = Instant::now();

    for i in 0..iterations {
        // Write
        manager.write_config_with_backup(&config_path, &config).unwrap();

        // Read
        let _read_config = manager.read_config(&config_path).unwrap();

        // Modify slightly
        config.custom_instructions = Some(vec![format!("Test instructions v{}", i)]);
    }

    let duration = start.elapsed();
    let avg_duration = duration / iterations;

    println!("10 parse-write cycles took: {:?} (avg: {:?} per cycle)", duration, avg_duration);

    // Each cycle should be fast (write + read)
    assert!(avg_duration.as_millis() < 20, "Average cycle took {:?}, expected < 20ms", avg_duration);
}
