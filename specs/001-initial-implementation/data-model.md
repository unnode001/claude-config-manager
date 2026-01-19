# Data Model: Claude Config Manager

**Feature**: 001-initial-implementation
**Last Updated**: 2025-01-19

## Overview

This document describes the core data structures and types used throughout the Claude Config Manager application. All types are defined in the Core Library (`crates/core`) and are used by both CLI and future GUI frontends.

## Core Types

### ClaudeConfig

Represents a complete Claude Code configuration file.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcpServers: Option<HashMap<String, McpServer>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowedPaths: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<HashMap<String, Skill>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub customInstructions: Option<Vec<String>>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}
```

**Fields**:
- `mcpServers`: Map of server name to server configuration
- `allowedPaths`: List of allowed filesystem paths
- `skills`: Map of skill name to skill configuration
- `customInstructions`: User-defined custom instructions
- `other`: Catch-all for unknown fields (forward-compatibility)

### McpServer

Represents a single MCP (Model Context Protocol) server configuration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpServer {
    pub enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}
```

**Fields**:
- `enabled`: Whether the server is enabled
- `args`: Command-line arguments for the server
- `env`: Environment variables for the server

**Validation Rules**:
- `enabled` must be boolean
- `args` must be array of strings if present
- `env` must be object with string values if present

### Skill

Represents a Claude Code skill configuration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    pub enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}
```

**Fields**:
- `enabled`: Whether the skill is enabled
- `parameters`: Skill-specific parameters (flexible types)

## Configuration Layer Types

### ConfigLayer

Represents a single configuration layer in the hierarchy.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigLayer {
    System { path: PathBuf, config: ClaudeConfig },
    Project { path: PathBuf, config: ClaudeConfig },
    Session { config: ClaudeConfig },
}
```

**Variants**:
- `System`: Global configuration from `~/.claude/config.json`
- `Project`: Project-specific configuration from `<project>/.claude/config.json`
- `Session`: Temporary overrides (not persisted)

**Methods**:
```rust
impl ConfigLayer {
    pub fn path(&self) -> Option<&PathBuf>;
    pub fn config(&self) -> &ClaudeConfig;
    pub fn layer_type(&self) -> LayerType;
}
```

### LayerType

Enum for easy comparison of layer types.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    System,
    Project,
    Session,
}
```

### MergedConfig

Represents the result of merging multiple configuration layers.

```rust
#[derive(Debug, Clone)]
pub struct MergedConfig {
    pub layers: Vec<ConfigLayer>,
    pub merged: ClaudeConfig,
    pub source_map: SourceMap,
}
```

**Fields**:
- `layers`: Original layers in merge order (lowest to highest priority)
- `merged`: Final merged configuration
- `source_map`: Tracks which layer each value came from

### SourceMap

Tracks the origin of each configuration value.

```rust
#[derive(Debug, Clone)]
pub struct SourceMap {
    // Key path -> originating layer
    // Example: "mcpServers.npx.enabled" -> LayerType::System
    pub sources: HashMap<String, LayerSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerSource {
    pub layer_type: LayerType,
    pub path: Option<PathBuf>,
}
```

## Management Types

### ConfigManager

Main API for configuration operations.

```rust
pub struct ConfigManager {
    global_config_path: PathBuf,
    project_cache: HashMap<PathBuf, ConfigLayer>,
    backup_manager: BackupManager,
}
```

**Responsibilities**:
- Load and cache configuration files
- Merge configurations from multiple layers
- Write configurations with atomic operations and backups
- Validate configurations before writing

### McpManager

Manages MCP server configurations.

```rust
pub struct McpManager {
    config_manager: ConfigManager,
}
```

**Responsibilities**:
- List servers (with scope filtering)
- Enable/disable servers
- Add/remove servers
- Update server arguments and environment

### ProjectManager

Manages project discovery and configuration.

```rust
pub struct ProjectManager {
    config_manager: ConfigManager,
}

impl ProjectManager {
    pub fn scan_directory(&self, path: &Path) -> Result<Vec<Project>, Error>;
    pub fn detect_project(&self, current_dir: &Path) -> Result<Option<PathBuf>, Error>;
}
```

### BackupManager

Manages backup files.

```rust
pub struct BackupManager {
    retention_count: usize,
}

impl BackupManager {
    pub fn create_backup(&self, config_path: &Path) -> Result<PathBuf, Error>;
    pub fn list_backups(&self, config_path: &Path) -> Result<Vec<BackupInfo>, Error>;
    pub fn restore_backup(&self, backup_path: &Path, config_path: &Path) -> Result<(), Error>;
    pub fn cleanup_old_backups(&self, config_path: &Path) -> Result<usize, Error>;
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub backup_path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub size: u64,
}
```

## CLI-Specific Types

### CliOutput

Output formatting for CLI commands.

```rust
pub enum CliOutput {
    Table(TableOutput),
    Json(JsonOutput),
    Human(String),
}
```

### ConfigDiff

Represents differences between two configurations.

```rust
#[derive(Debug, Clone)]
pub struct ConfigDiff {
    pub additions: HashMap<String, serde_json::Value>,
    pub removals: HashMap<String, serde_json::Value>,
    pub modifications: HashMap<String, ValueChange>,
}

#[derive(Debug, Clone)]
pub struct ValueChange {
    pub old: serde_json::Value,
    pub new: serde_json::Value,
}
```

## Error Types

### ConfigError

All configuration-related errors using `thiserror`.

```rust
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Invalid JSON in config file '{path}': {source}")]
    InvalidJson {
        path: PathBuf,
        #[source] serde_json::Error,
    },

    #[error("Config validation failed: {message}")]
    ValidationFailed { message: String },

    #[error("Filesystem error on '{path}': {source}")]
    Filesystem {
        path: PathBuf,
        #[source] std::io::Error,
    },

    #[error("Backup failed for '{path}': {reason}")]
    BackupFailed { path: PathBuf, reason: String },

    #[error("Permission denied: '{path}'")]
    PermissionDenied { path: PathBuf },

    #[error("Merge failed: {message}")]
    MergeError { message: String },
}
```

## Validation Types

### ValidationRule

Represents a single validation rule.

```rust
pub trait ValidationRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<(), ValidationError>;
}

pub struct ValidationError {
    pub path: String,
    pub message: String,
}
```

### Built-in Rules

```rust
pub struct McpServersRule;
impl ValidationRule for McpServersRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<(), ValidationError> {
        // Validate mcpServers structure
    }
}

pub struct AllowedPathsRule;
impl ValidationRule for AllowedPathsRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<(), ValidationError> {
        // Validate allowedPaths is array of strings
    }
}
```

## Type Conversions

### Serde JSON Integration

All configuration types support serialization to/from JSON:

```rust
impl ClaudeConfig {
    pub fn from_json(json: &str) -> Result<Self, ConfigError>;
    pub fn to_json_pretty(&self) -> Result<String, ConfigError>;
    pub fn to_json_compact(&self) -> Result<String, ConfigError>;
}
```

### Path Handling

Use `camino::Utf8Path` for cross-platform path handling:

```rust
use camino::Utf8PathBuf;

pub struct ConfigPaths {
    pub global: Utf8PathBuf,
    pub project: Option<Utf8PathBuf>,
}
```

## Performance Considerations

### Lazy Loading

Configurations are loaded on-demand and cached:

```rust
lazy_static! {
    static ref CONFIG_CACHE: RwLock<HashMap<PathBuf, ClaudeConfig>> =
        RwLock::new(HashMap::new());
}
```

### Clone Optimization

Use `Arc` for shared data to minimize clones:

```rust
use std::sync::Arc;

pub struct ConfigManager {
    global_config: Arc<ClaudeConfig>,
}
```

## Summary

This data model provides:

✅ **Type Safety**: Rust's type system prevents entire classes of bugs
✅ **Validation**: All configs validated before use
✅ **Immutability**: Configs are cloned on modification (functional style)
✅ **Serialization**: Easy JSON serialization/deserialization
✅ **Error Handling**: Comprehensive error types with clear messages
✅ **Testing**: All types are easily testable
✅ **Documentation**: Rustdoc on all public types

**Next**: See [quickstart.md](./quickstart.md) for usage examples.
