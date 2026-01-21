# Changelog

All notable changes to Claude Config Manager will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-21

### Added

#### Core Features
- Multi-level configuration management (global/project)
- Configuration merging with smart strategies
- Configuration validation with actionable error messages
- Automatic backup system before any write operation
- Atomic write operations to prevent data corruption
- Project discovery and scanning
- Configuration search across all levels
- Import/export functionality (JSON/TOML)
- History management with restore capability

#### MCP Server Management
- List all MCP servers (global and project-scoped)
- Add new MCP servers with command, args, and environment variables
- Remove MCP servers
- Enable/disable MCP servers
- Show detailed server information

#### Configuration Commands
- `config get` - View configuration (table/JSON format)
- `config set` - Set configuration values by key path
- `config diff` - Compare global vs project configuration
- `config import` - Import configuration from file
- `config export` - Export configuration to file

#### Project Commands
- `project scan` - Scan directory for `.claude` projects
- `project list` - List all discovered projects
- `project config` - Show project configuration details

#### Search Command
- Search by key or value
- Case-insensitive search
- Regex pattern support
- Source tracking (global/project origin)

#### History Commands
- `history list` - List all backups with metadata
- `history restore` - Restore from backup
- Automatic cleanup (retains 10 backups by default)

#### Safety Features
- Pre-write validation
- Automatic backup creation
- Atomic file operations
- Clear error messages with suggestions

#### Developer Features
- 241 tests (unit + integration)
- 100% test coverage on core functionality
- Zero compiler warnings
- Cross-platform support (Windows, macOS, Linux)
- Performance-optimized (<10ms config parsing)

### Performance

- CLI startup: ~50ms (target: <100ms)
- Config parsing: ~1-3ms (target: <10ms)
- Config write: ~5-15ms (target: <50ms)
- MCP server list: ~5-10ms (target: <50ms)

### Documentation

- Comprehensive README with quick start guide
- Command reference for all CLI commands
- Contributing guidelines
- Architecture documentation
- Task tracking (175 tasks completed)

[Unreleased]: https://github.com/your-org/claude-config-manager/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/claude-config-manager/releases/tag/v0.1.0
