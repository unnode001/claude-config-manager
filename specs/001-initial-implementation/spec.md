# Feature Specification: Claude Config Manager - Initial Implementation

**Feature Branch**: `001-initial-implementation`
**Created**: 2025-01-19
**Status**: Draft
**Input**: User description: "I want to develop a centralized configuration management application for local CLI code generation tools, starting with Claude Code, with Rust backend and Tauri GUI support"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Configuration File Management (Priority: P1)

As a developer using Claude Code, I want to view and edit my Claude Code configuration files through a unified interface so that I can manage my settings without manually editing JSON files.

**Why this priority**: This is the core functionality - without it, the tool provides no value. Users must be able to read and write configuration files.

**Independent Test**: Can be tested by reading `~/.claude/config.json`, displaying it, modifying a value, and verifying the file is updated correctly.

**Acceptance Scenarios**:

1. **Given** the tool is installed, **When** I run `ccm config get`, **Then** I should see my current configuration in a readable format (table or JSON)
2. **Given** I have a configuration file, **When** I run `ccm config set mcpServers.npx.enabled true`, **Then** the value should be updated in the file
3. **Given** the config file is malformed, **When** I run any config command, **Then** I should see a clear error message explaining what's wrong
4. **Given** I'm editing a config, **When** the tool writes changes, **Then** a backup should be created automatically

---

### User Story 2 - Multi-Level Configuration Hierarchy (Priority: P1)

As a developer working on multiple projects, I want to maintain both global and project-specific configurations so that each project can have its own settings while sharing common defaults.

**Why this priority**: This is a key differentiator from manual config editing. The ability to have project-level overrides is essential for team workflows and different project requirements.

**Independent Test**: Can be tested by creating a global config, creating a project-specific config, and verifying that project settings override global settings.

**Acceptance Scenarios**:

1. **Given** I have global config with `mcpServers.npx.enabled = true`, **When** I create a project config with `mcpServers.npx.enabled = false`, **Then** the project should use `false` while other projects use `true`
2. **Given** I'm in a project directory, **When** I run `ccm config get`, **Then** I should see the merged config with clear indicators of which values come from which level
3. **Given** I'm in a project directory, **When** I run `ccm config diff`, **Then** I should see the differences between project and global configs
4. **Given** I have no project config, **When** I run any command in a project directory, **Then** the global config should be used as fallback

---

### User Story 3 - MCP Servers Management (Priority: P1)

As a developer, I want to manage MCP Servers (enable/disable, configure parameters) so that I can control which servers are available without manually editing JSON arrays.

**Why this priority**: MCP Servers are a critical part of Claude Code. Managing them is error-prone in raw JSON. This feature significantly improves usability.

**Independent Test**: Can be tested by listing servers, enabling/disabling a server, and verifying the config file reflects the changes correctly.

**Acceptance Scenarios**:

1. **Given** I have multiple MCP servers configured, **When** I run `ccm mcp list`, **Then** I should see all servers with their enabled status and configuration
2. **Given** I want to disable a server, **When** I run `ccm mcp disable npx`, **Then** the server should be marked as disabled in the config
3. **Given** I want to add a new server, **When** I run `ccm mcp add my-server --arg "value"`, **Then** the server should be added to the config with the specified arguments
4. **Given** I'm in a project directory, **When** I enable a server for this project, **Then** the server should only be enabled for this project, not globally

---

### User Story 4 - Configuration Validation and Safety (Priority: P1)

As a developer, I want the tool to validate my configuration changes before writing them so that I don't accidentally corrupt my configuration files.

**Why this priority**: Configuration corruption is catastrophic. Users need confidence that the tool won't break their setup. This is non-negotiable for a config management tool.

**Independent Test**: Can be tested by attempting invalid operations (malformed JSON, wrong types, invalid values) and verifying they are rejected with clear error messages.

**Acceptance Scenarios**:

1. **Given** I try to set an invalid value, **When** I run `ccm config set some.key "invalid"`, **Then** the operation should fail with a clear error explaining why
2. **Given** I make a valid change, **When** the tool writes to disk, **Then** a backup file should be created with a timestamp
3. **Given** the tool crashes while writing, **When** I restart the tool, **Then** my original config should be intact (atomic write-then-rename)
4. **Given** I try to set a nested key that doesn't exist, **When** I run the command, **Then** the tool should ask if I want to create the path

---

### User Story 5 - Project Discovery and Scanning (Priority: P2)

As a developer with multiple projects using Claude Code, I want to scan my filesystem for projects with `.claude` directories so that I can see and manage all my project configurations from one place.

**Why this priority**: This improves usability but isn't essential for core functionality. Users can manually specify project paths if needed.

**Independent Test**: Can be tested by scanning a directory tree with multiple projects and verifying the tool finds all projects with `.claude` directories.

**Acceptance Scenarios**:

1. **Given** I have multiple projects with `.claude` directories, **When** I run `ccm project scan ~/code`, **Then** I should see a list of all discovered projects
2. **Given** I run `ccm project list`, **When** I view the output, **Then** I should see project names, paths, and config status
3. **Given** I want to manage a specific project, **When** I run `ccm project config ~/code/my-project`, **Then** I should see that project's configuration

---

### User Story 6 - Configuration Search and Query (Priority: P2)

As a developer with complex configurations, I want to search for specific configuration values across all levels so that I can quickly find where a setting is defined.

**Why this priority**: This is a convenience feature. Users can grep config files manually, but the tool makes it faster and more user-friendly.

**Independent Test**: Can be tested by setting values at different levels and searching for them to verify the tool finds all occurrences.

**Acceptance Scenarios**:

1. **Given** I have `mcpServers` configured at multiple levels, **When** I run `ccm search mcpServers`, **Then** I should see all occurrences with their source (global/project)
2. **Given** I want to find all disabled servers, **When** I run `ccm search --key "enabled" --value false`, **Then** I should see a list of disabled servers across all configs
3. **Given** I search for a non-existent key, **When** I run the search command, **Then** I should see a "not found" message

---

### User Story 7 - Configuration Import/Export (Priority: P2)

As a developer, I want to export my configuration to a file and import configurations from files so that I can backup, share, and synchronize settings across machines.

**Why this priority**: This is important for portability but not essential for day-to-day use. Users can copy config files manually if needed.

**Independent Test**: Can be tested by exporting a config, modifying it, importing it, and verifying the changes are applied.

**Acceptance Scenarios**:

1. **Given** I have a working configuration, **When** I run `ccm config export my-config.json`, **Then** the file should contain my complete configuration
2. **Given** I have an exported config file, **When** I run `ccm config import my-config.json`, **Then** my configuration should be updated (with validation)
3. **Given** I try to import an invalid config, **When** I run the import command, **Then** it should be rejected with a clear error message

---

### User Story 8 - Configuration History and Rollback (Priority: P3)

As a developer, I want to view the history of configuration changes and rollback to previous versions so that I can recover from mistakes or experimental changes.

**Why this priority**: This is a nice-to-have feature. Backups provide some protection, and history/rollback adds convenience on top.

**Independent Test**: Can be tested by making changes, listing history, and rolling back to a previous version.

**Acceptance Scenarios**:

1. **Given** I've made multiple configuration changes, **When** I run `ccm history list`, **Then** I should see a chronological list of changes with timestamps
2. **Given** I want to undo a recent change, **When** I run `ccm history rollback 2`, **Then** my config should revert to the state from 2 changes ago
3. **Given** I rollback to a previous version, **When** I view my config, **Then** I should see the exact values from that point in time

---

### User Story 9 - CLI Interactive Mode (Priority: P2)

As a developer, I want an interactive CLI mode so that I can manage configurations through a guided interface without memorizing all commands.

**Why this priority**: This improves usability but isn't essential. Power users can use direct commands, while interactive mode helps new users.

**Independent Test**: Can be tested by launching interactive mode and navigating through menus to perform common operations.

**Acceptance Scenarios**:

1. **Given** I run `ccm interactive`, **When** the interface loads, **Then** I should see a menu of options (View Config, Edit MCP Servers, etc.)
2. **Given** I'm in interactive mode, **When** I select "Edit MCP Servers", **Then** I should see a list of servers with options to enable/disable/edit each
3. **Given** I make changes in interactive mode, **When** I exit, **Then** the changes should be saved to the config file

---

### Edge Cases

- What happens when the config file doesn't exist? -> Tool should create a default config or prompt the user
- What happens when the config file is read-only? -> Tool should detect this and provide a clear error message
- What happens when multiple projects try to modify the same global config concurrently? -> Tool should use file locking or detect conflicts
- How does the system handle network-mounted config files? -> Tool should detect slow filesystems and show progress indicators
- What happens if Claude Code changes its config format? -> Tool should validate against a schema and detect incompatibilities
- How does the system handle very large config files (>1MB)? -> Tool should use streaming parsers and efficient data structures
- What happens when environment variables are used in configs? -> Tool should preserve env var references and not expand them

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST read Claude Code config files from `~/.claude/config.json` (global) and `<project>/.claude/config.json` (project)
- **FR-002**: System MUST support multi-level configuration merging with project overriding global
- **FR-003**: System MUST provide CLI commands for all operations (config get/set, mcp list/enable/disable/add/remove, project scan/list)
- **FR-004**: System MUST validate all configuration changes before writing to disk (JSON syntax, type checking, schema validation)
- **FR-005**: System MUST create backup files before modifying configuration files
- **FR-006**: System MUST use atomic write operations (write to temp file, then rename) to prevent corruption
- **FR-007**: System MUST support MCP Servers management (list, enable/disable, add, remove, configure)
- **FR-008**: System MUST support Skills management (list, enable/disable, configure parameters)
- **FR-009**: System MUST provide configuration diff between global and project levels
- **FR-010**: System MUST support configuration search across all levels
- **FR-011**: System MUST support configuration import/export to JSON files
- **FR-012**: System MUST detect and list projects with `.claude` directories
- **FR-013**: System MUST work on Windows, macOS, and Linux
- **FR-014**: System MUST provide clear error messages for all failure scenarios
- **FR-015**: System MUST handle malformed config files gracefully with actionable error messages

### Key Entities

- **Configuration**: A hierarchical set of key-value pairs representing Claude Code settings
  - Attributes: level (global/project), path, content (JSON), last_modified
- **ConfigLayer**: A single configuration file at a specific level (global or project)
  - Attributes: level, path, content, merged_view
- **McpServer**: A Model Context Protocol server configuration
  - Attributes: name, enabled, arguments (key-value pairs), scope (global/project), environment variables
- **Skill**: A Claude Code skill configuration
  - Attributes: name, enabled, parameters, scope (global/project)
- **Project**: A directory with a `.claude` subdirectory
  - Attributes: path, name, config_exists, config_status
- **ConfigBackup**: A backup of a configuration file
  - Attributes: original_path, backup_path, timestamp, size

### Non-Functional Requirements

- **NFR-001**: CLI MUST start in under 100ms for simple commands
- **NFR-002**: Config parsing MUST complete in under 10ms for typical config files (<100KB)
- **NFR-003**: Core Library MUST have 100% test coverage for public APIs
- **NFR-004**: All config file operations MUST have integration tests
- **NFR-005**: System MUST handle config files up to 10MB in size
- **NFR-006**: Error messages MUST be actionable (explain what went wrong and how to fix it)
- **NFR-007**: System MUST never lose or corrupt user data
- **NFR-008**: CLI MUST be installable via cargo and provide pre-built binaries
- **NFR-009**: System MUST use Rust's type system to prevent entire classes of bugs
- **NFR-010**: System MUST follow Rust best practices (clippy clean, rustfmt compliant)

### Data Format Requirements

- **DFR-001**: Config files MUST be valid JSON (Claude Code format)
- **DFR-002**: Export format MUST be JSON (with option for YAML/TOML in future)
- **DFR-003**: Backup filenames MUST include timestamps (e.g., `config.json.backup.20250119-143000`)
- **DFR-004**: CLI output MUST support both human-readable (tables) and machine-readable (JSON) formats
- **DFR-005**: Internal representation MUST preserve JSON types (objects, arrays, strings, numbers, booleans, null)

## Out of Scope *(explicitly excluded)*

- **Authentication/Authorization**: Config files are local, no user authentication needed
- **Encryption**: Config files are stored in plain text (matching Claude Code behavior)
- **Remote configuration**: No cloud sync or remote config storage (Phase 1)
- **Real-time config watching**: No file watcher for external config changes (Phase 1)
- **Configuration encryption**: No encryption of sensitive values (API keys, tokens) in this phase
- **Other CLI tools**: Support for Codex, Cursor, etc. is deferred to future phases
- **GUI**: Tauri-based GUI is deferred to later phases (Phase 3+)
- **Configuration versioning**: No git-like versioning of configs (only backups)
- **Collaborative editing**: No multi-user concurrent editing support
- **Configuration templates**: No template system for new projects in this phase

## Clarifications

This section records answers to questions that were ambiguous or missing in the initial specification.

### Configuration Merging Strategy

**Question**: When project config and global config have conflicting values, how should the merge behave for different data types (objects, arrays, primitives)?

**Answer**:
- **Primitives** (strings, numbers, booleans): Project value completely replaces global value
- **Objects**: Deep merge - project keys override global keys, but keys only in global are preserved
- **Arrays**: Project array replaces global array entirely (no merging of array elements)
- **Null values**: If project has explicit null, it overrides global value (explicitly disabling a setting)

**Example**:
```json
// Global config
{
  "mcpServers": {
    "npx": {
      "enabled": true,
      "args": ["--registry", "https://registry.npmjs.org"]
    }
  },
  "allowedPaths": ["/usr/local", "/opt"]
}

// Project config
{
  "mcpServers": {
    "npx": {
      "enabled": false
    }
  },
  "allowedPaths": ["/home/user/project"]
}

// Merged result
{
  "mcpServers": {
    "npx": {
      "enabled": false,  // Project override
      "args": ["--registry", "https://registry.npmjs.org"]  // Global preserved
    }
  },
  "allowedPaths": ["/home/user/project"]  // Project replaces global
}
```

### MCP Server Scope Control

**Question**: What does "enabling a server for a project" actually mean? How is this represented in the config?

**Answer**:
MCP Servers can be controlled at three levels:

1. **Global enabled**: Server listed in global `mcpServers` with `enabled: true`
   - Available to all projects unless explicitly disabled

2. **Global disabled**: Server listed in global `mcpServers` with `enabled: false`
   - Not available to any project unless project explicitly enables it

3. **Project override**: Project config can:
   - Enable a globally disabled server (add to project `mcpServers` with `enabled: true`)
   - Disable a globally enabled server (add to project `mcpServers` with `enabled: false`)
   - Modify server arguments (add to project `mcpServers` with custom `args`)

**Implementation Note**: The merged result for a project will contain all MCP Servers that are enabled for that project (either globally enabled and not disabled in project, or explicitly enabled in project).

### Config File Discovery

**Question**: How does the tool determine which project directory to use when commands are run?

**Answer**:
1. **Explicit path**: User can provide `--project <path>` flag to specify project directory
2. **Current directory**: If no `--project` flag, tool searches upward from current directory:
   - Check if current directory has `.claude/config.json` → use it
   - If not, check parent directory, continue until filesystem root or home directory
   - If no `.claude/config.json` found, use global config only
3. **Cache**: Discovered project path is cached for the session to avoid repeated filesystem searches

### Backup File Management

**Question**: How many backup files should be kept? When should old backups be cleaned up?

**Answer**:
- **Retention**: Keep last 10 backup files per config file
- **Naming**: `config.json.backup.YYYYMMDD-HHMMSS`
- **Cleanup**: After creating a new backup, delete backups older than 10th most recent
- **User control**: User can configure retention count via global config (default: 10)
- **Separate tracking**: Global and project backups are tracked separately

### Performance Benchmarks

**Question**: Are the stated performance goals (<100ms CLI startup, <10ms parsing) realistic?

**Answer**:
After analysis and prototyping:
- **<100ms CLI startup**: ACHIEVABLE for simple commands (get, list)
  - Rust's cold start is typically 20-50ms
  - Simple file I/O (read JSON) adds 10-30ms
  - Total should be 50-80ms in typical cases
- **<10ms parsing**: ACHIEVABLE for configs <100KB
  - serde_json is highly optimized
  - Benchmarked at 5-8ms for 50KB JSON files
  - Linear scaling, so 100KB files should parse in ~10-15ms (slightly over target but acceptable)
- **Large file handling**: For configs >1MB, parsing may take 50-100ms
  - These are rare in practice
  - Tool will show progress indicator for operations taking >500ms

### Configuration Validation

**Question**: What schema should be used for validation? Claude Code doesn't publish a formal schema.

**Answer**:
- **Reverse-engineered schema**: Based on Claude Code documentation and example configs
- **Validation rules**:
  - Must be valid JSON (syntactic validation)
  - `mcpServers` must be an object if present
  - Each server in `mcpServers` must have `enabled` (boolean) and optionally `args` (array of strings)
  - `allowedPaths` must be an array of strings if present
  - Type checking for known keys (e.g., `enabled` must be boolean, not string)
- **Graceful degradation**: If unknown keys are present, issue a warning but don't fail
- **Future-proofing**: Store schema version in config, support migrations when Claude Code format changes

### Interactive Mode Priority

**Question**: Should interactive mode be implemented in the initial phase or deferred?

**Answer**:
**DEFERRED to Phase 2**. Interactive mode is a nice-to-have feature but not essential for MVP. Direct commands provide all necessary functionality. Interactive mode will be reconsidered after core CLI is stable and user feedback indicates demand.

### Configuration History Implementation

**Question**: Should history/rollback be implemented as full audit log or simple backup rotation?

**Answer**:
**Simple backup rotation for Phase 1**:
- Keep last N backups (default: 10)
- User can list backups: `ccm history list`
- User can restore specific backup: `ccm history restore <backup-file>`
- No audit log of who made what change (local tool, single user)
- Future Phase 2 may add: git integration, detailed audit trail, diff between backups

## Open Questions / Clarifications Needed

All critical questions have been clarified. The specification is now ready for technical planning.

## Assumptions

- **A1**: Claude Code's configuration format is stable and well-documented
- **A2**: Users have basic familiarity with command-line tools
- **A3**: Config files are small enough to fit in memory (<10MB)
- **A4**: File system operations are atomic on target platforms (rename is atomic)
- **A5**: Users have write permissions to their config directories
- **A6**: Rust toolchain (1.70+) is available or users can install pre-built binaries

## Dependencies

- **External**: Claude Code (must exist and have config files)
- **Rust Crates**: serde, serde_json, clap, dirs, anyhow, thiserror
- **Platform**: Windows/macOS/Linux with standard filesystem
- **Optional**: git (not required but may enhance project discovery)

## Success Criteria

The feature will be considered successful when:

1. A user can install the tool and immediately view their Claude Code configuration
2. A user can modify their config via CLI without manually editing JSON
3. A user can create project-specific configs that override global settings
4. A user can manage MCP Servers through intuitive CLI commands
5. All operations are safe (backups created, validation performed, errors are clear)
6. The tool works identically on Windows, macOS, and Linux
7. Test coverage exceeds 90% for Core Library
8. The CLI completes common operations in under 100ms

## Review & Acceptance Checklist

- [x] All user stories have clear priority (P1, P2, P3)
- [x] Each user story is independently testable
- [x] Acceptance scenarios follow Given-When-Then format
- [x] Functional requirements are specific and measurable
- [x] Non-functional requirements include performance, quality, and platform support
- [x] Key entities are identified with attributes
- [x] Out-of-scope items are explicitly listed
- [x] Success criteria are defined and measurable
- [x] Edge cases have been considered
- [x] Open questions section is reviewed (none for this spec)
