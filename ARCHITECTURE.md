# Architecture

This document describes the architecture of Claude Config Manager.

## Overview

Claude Config Manager follows a **three-layer architecture** with clear separation between business logic and user interface.

```
┌─────────────────────────────────────────────────────────┐
│                   Frontend Layer                        │
│  ┌────────────────┐           ┌────────────────┐        │
│  │  CLI App       │           │  GUI App       │        │
│  │  (clap)        │           │  (Tauri)       │        │
│  │  crates/cli/   │           │  crates/tauri/ │        │
│  └────────────────┘           └────────────────┘        │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    Core Library                         │
│              crates/core/src/                            │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │   Config   │  │    MCP     │  │  Project   │        │
│  │ Management │  │ Management │  │ Discovery  │        │
│  └────────────┘  └────────────┘  └────────────┘        │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐        │
│  │   Backup   │  │   Search   │  │ Validation │        │
│  │   System   │  │  Function  │  │   Engine   │        │
│  └────────────┘  └────────────┘  └────────────┘        │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    File System                          │
│              Configuration Files & Backups               │
└─────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Core Library First

All business logic lives in `crates/core/`, which has no dependencies on frontend libraries (clap, tauri, etc.). This allows:

- Easy addition of new frontends (GUI, web, etc.)
- Reusable library for other projects
- Clear separation of concerns
- Independent testing

### 2. Frontend Agnosticism

The core library uses:

- `camino::UtfPathBuf` for cross-platform paths
- `thiserror` for error types
- `serde` for serialization
- No CLI/GUI-specific types

### 3. Safety & Reliability

- **Atomic Writes**: All writes use atomic rename pattern
- **Automatic Backups**: Every write creates a backup first
- **Validation**: All inputs validated before applying
- **Clear Errors**: Actionable error messages with suggestions

### 4. Test-Driven Development

- 241 tests (unit + integration)
- 100% coverage on critical paths
- TDD workflow for new features

## Module Organization

### Core Library (`crates/core/src/`)

```
src/
├── lib.rs              # Public API exports
├── error.rs            # Error types (ConfigError)
├── types.rs            # Shared types (ClaudeConfig, McpServer, etc.)
├── config/             # Configuration management
│   ├── mod.rs          # Configuration types
│   ├── manager.rs      # ConfigManager (read/write/merge)
│   ├── merge.rs        # Configuration merging logic
│   ├── layer.rs        # Configuration layers
│   └── validation.rs   # Validation rules
├── mcp/                # MCP server management
│   ├── mod.rs          # MCP types
│   └── manager.rs      # McpManager
├── backup/             # Backup system
│   └── mod.rs          # BackupManager
├── search/             # Search functionality
│   └── mod.rs          # ConfigSearcher
├── project/            # Project discovery
│   └── mod.rs          # ProjectScanner
├── import_export.rs    # Import/Export functionality
└── paths.rs            # Path resolution utilities
```

### CLI Application (`crates/cli/src/`)

```
src/
├── main.rs             # CLI entry point
├── commands/           # Command handlers
│   ├── mod.rs          # Command exports
│   ├── config.rs       # config get/set/diff/import/export
│   ├── mcp.rs          # mcp list/add/remove/enable/disable
│   ├── history.rs      # history list/restore
│   ├── project.rs      # project scan/list
│   └── search.rs       # search command
├── key_path.rs         # Key path parsing (e.g., "mcpServers.npx.enabled")
└── output/             # Output formatting
    ├── mod.rs          # Formatter exports
    ├── table.rs        # Table output
    └── json.rs         # JSON output
```

## Data Flow

### Reading Configuration

```
User Command: ccm config get
       │
       ▼
CLI Parser (clap)
       │
       ▼
Command Handler (commands/config.rs)
       │
       ├─> ConfigManager::get_global_config()
       │         │
       │         ├─> paths::get_global_config_path()
       │         │         │
       │         │         └─> dirs::config_dir() / "claude/config.json"
       │         │
       │         ├─> fs::read_to_string()
       │         │
       │         ├─> serde_json::from_str()
       │         │
       │         └─> Returns ClaudeConfig
       │
       ├─> Output Formatter (table.rs)
       │         │
       │         └─> Prints to stdout
       │
       └─> Returns Ok(())
```

### Writing Configuration

```
User Command: ccm config set customInstructions "test"
       │
       ▼
CLI Parser (clap)
       │
       ▼
Command Handler (commands/config.rs)
       │
       ├─> key_path::parse("customInstructions")
       │         │
       │         └─> Returns Vec<&str>
       │
       ├─> ConfigManager::get_global_config()
       │
       ├─> key_path::set_value() // Modifies config in memory
       │
       ├─> validate_config() // Validates before write
       │
       ├─> BackupManager::create_backup() // Creates backup first
       │         │
       │         └─> Atomic copy to backup directory
       │
       ├─> ConfigManager::write_config_with_backup()
       │         │
       │         ├─> Write to temp file
       │         ├─> Atomic rename (overwrites original)
       │         └─> Returns on success
       │
       └─> Returns Ok(())
```

### Configuration Merging

```
User Request: Get effective config for project
       │
       ▼
ConfigManager::get_merged_config(project_path)
       │
       ├─> read_config(global_path) // Returns global_config
       │
       ├─> read_config(project_path) // Returns project_config
       │
       ├─> merge_configs(&global_config, &project_config)
       │         │
       │         ├─> Objects (HashMap): Deep merge
       │         │   - Add new keys from override
       │         │   - Replace existing keys
       │         │
       │         ├─> Arrays (Vec): Replace
       │         │   - Override replaces base
       │         │
       │         ├─> Primitives: Replace
       │         │
       │         └─> Returns merged ClaudeConfig
       │
       └─> Returns merged config
```

## Key Algorithms

### Configuration Merge Strategy

```rust
fn merge_configs(base: &ClaudeConfig, override: &ClaudeConfig) -> ClaudeConfig {
    ClaudeConfig {
        // Deep merge: Add/replace servers
        mcp_servers: merge_hashmaps(
            base.mcp_servers,
            override.mcp_servers
        ),

        // Replace: Override replaces base entirely
        allowed_paths: override.allowed_paths
            .or(base.allowed_paths),

        // Deep merge: Add/replace skills
        skills: merge_hashmaps(
            base.skills,
            override.skills
        ),

        // Replace: Override replaces base
        custom_instructions: override.custom_instructions
            .or(base.custom_instructions),

        // Preserve unknown fields
        unknown: merge_hashmaps(
            base.unknown,
            override.unknown
        ),
    }
}
```

### Backup Naming

```rust
// Format: <filename>_<timestamp>[_<counter>].<ext>
// Example: config_20250120_143022_123.json

let timestamp = Utc::now().format("%Y%m%d_%H%M%S%.6f");
let mut backup_name = format!("{}_{}.{}", file_stem, timestamp, extension);

// Handle collisions: add counter if needed
while backup_path.exists() {
    counter += 1;
    backup_name = format!("{}_{}_{}.{}", file_stem, timestamp, counter, extension);
}
```

### Atomic Write Pattern

```rust
// 1. Write to temporary file
let temp_path = format!("{}.tmp", file_path);
fs::write(&temp_path, contents)?;

// 2. Atomic rename (overwrites target)
fs::rename(&temp_path, &file_path)?;

// On success: atomic swap (original preserved until rename succeeds)
// On failure: temp file left behind, original intact
```

## Error Handling

All errors use `ConfigError` enum:

```rust
pub enum ConfigError {
    NotFound(PathBuf),                    // File not found
    InvalidJson(String),                  // Parse error with location
    ValidationFailed(String, String),     // (rule, suggestion)
    Filesystem(String, PathBuf, io::Error), // (operation, path, cause)
    BackupFailed(PathBuf),                // Backup creation failed
    PermissionDenied(PathBuf),            // Read/write permission error
}
```

Error messages provide:
- What went wrong
- Where it happened
- How to fix it

Example:
```
Error: Failed to read configuration file

Path: ~/.claude/config.json
Cause: Permission denied

Suggestion: Check file permissions with:
  ls -la ~/.claude/config.json

Fix with:
  chmod 644 ~/.claude/config.json
```

## Testing Strategy

### Unit Tests

- Located alongside source code in `src/`
- Test individual functions and methods
- Use mock data/fixtures
- Fast (<1ms per test)

### Integration Tests

- Located in `crates/core/tests/`
- Test real filesystem operations
- Use `tempfile` for isolation
- Slower but comprehensive

### CLI Tests

- Located in `crates/cli/tests/`
- Use `assert_cmd` for end-to-end testing
- Test full command flows
- Verify output formatting

## Performance Considerations

### Lazy Evaluation

- Configs read only when needed
- JSON parsed on demand
- Backups created only before writes

### Caching

- Project detection result cached
- Parsed configs cached in memory
- Backup list cached during command execution

### Optimization Targets

| Operation | Target | Actual |
|-----------|--------|--------|
| CLI startup | <100ms | ~50ms |
| Config parse | <10ms | ~1-3ms |
| Config write | <50ms | ~5-15ms |
| Merge (2 configs) | <5ms | <1ms |

## Future Enhancements

### Planned

- [ ] GUI application (Tauri)
- [ ] Interactive CLI mode
- [ ] Configuration templates
- [ ] Watch mode (monitor config changes)

### Possible

- [ ] Plugin system for custom validators
- [ ] Remote configuration sync
- [ ] Configuration versioning
- [ ] Support for other AI tools
