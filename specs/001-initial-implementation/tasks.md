# Tasks: Claude Config Manager - Initial Implementation

**Input**: Design documents from `/specs/001-initial-implementation/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/claude-config-spec.md

**Tests**: YES - TDD is mandatory per Constitution Principle IV

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3...)
- **Checkpoint**: Verification points after each phase
- **File paths**: Use repository structure from plan.md

---

## Phase 1: Project Setup (Shared Infrastructure)

**Purpose**: Initialize Rust workspace and development environment

### 1.1 Repository Setup

- [ ] **T001** Initialize Git repository with proper `.gitignore` (Rust-specific: target/, Cargo.lock, .idea/, etc.)
- [ ] **T002** Create workspace `Cargo.toml` with members: `crates/core`, `crates/cli`, `crates/tauri`
- [ ] **T003** [P] Set up GitHub Actions CI workflow (test on Windows/macOS/Linux)
- [ ] **T004** [P] Create LICENSE file (MIT) and README.md skeleton
- [ ] **T005** [P] Configure development tools:
  - rustfmt.toml with standard settings
  - clippy.toml with strict warnings
  - .cargo/config.toml for build optimizations

### 1.2 Core Library Skeleton

- [ ] **T006** Create `crates/core/Cargo.toml` with dependencies:
  - serde, serde_json, serde_toml
  - thiserror, anyhow
  - dirs, camino
  - tracing, tracing-subscriber
  - rstest (dev-dependency)
- [ ] **T007** Create `crates/core/src/lib.rs` with module declarations
- [ ] **T008** [P] Create `crates/core/src/error.rs` with ConfigError enum (thiserror)
- [ ] **T009** [P] Create `crates/core/src/types.rs` with shared types

### 1.3 CLI Skeleton

- [ ] **T010** Create `crates/cli/Cargo.toml` with dependencies:
  - clap (derive feature)
  - core workspace dependency
- [ ] **T011** Create `crates/cli/src/main.rs` with basic CLI structure
- [ ] **T012** [P] Create `crates/cli/src/commands/mod.rs`

**Checkpoint**: ✅ Project structure complete, CI configured, basic skeletons ready

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### 2.1 Configuration Types

- [ ] **T013** Define `ClaudeConfig` struct in `crates/core/src/config/mod.rs`
- [ ] **T014** Define `McpServer` struct in `crates/core/src/mcp/mod.rs`
- [ ] **T015** [P] Define `Skill` struct in `crates/core/src/skills/mod.rs`
- [ ] **T016** [P] Define `ConfigLayer` enum in `crates/core/src/config/layer.rs`
- [ ] **T017** Add serde Serialize/Deserialize derives to all config types
- [ ] **T018** [P] Write unit tests for config type serialization/deserialization

### 2.2 Error Handling

- [ ] **T019** Complete `ConfigError` enum with all variants:
  - NotFound, InvalidJson, ValidationFailed, Filesystem, BackupFailed, PermissionDenied
- [ ] **T020** Implement Display trait for all error variants
- [ ] **T021** [P] Add integration test for error messages (ensure clarity)

### 2.3 Configuration Validation

- [ ] **T022** Define `ValidationRule` trait in `crates/core/src/config/validation.rs`
- [ ] **T023** Implement `McpServersRule` validation
- [ ] **T024** [P] Implement `AllowedPathsRule` validation
- [ ] **T025** [P] Implement `SkillsRule` validation
- [ ] **T026** Write unit tests for each validation rule (positive + negative cases)

### 2.4 Backup System

- [ ] **T027** Create `crates/core/src/backup/mod.rs`
- [ ] **T028** Implement `BackupManager` with:
  - `create_backup()`: Create timestamped backup
  - `list_backups()`: List available backups
  - `cleanup_old_backups()`: Retention policy (default: 10)
- [ ] **T029** [P] Write unit tests for backup creation and cleanup
- [ ] **T030** [P] Write integration tests with real filesystem (using tempfile)

### 2.5 Configuration File I/O

- [ ] **T031** Implement `read_config()` in `crates/core/src/config/manager.rs`
- [ ] **T032** [P] Implement `write_config_with_backup()` using atomic rename pattern
- [ ] **T033** [P] Write unit tests for config reading (valid JSON, invalid JSON, missing file)
- [ ] **T034** [P] Write integration tests for atomic writes (simulate crash scenarios)

### 2.6 Configuration Merging

- [ ] **T035** Implement `merge_configs()` in `crates/core/src/config/merge.rs`
- [ ] **T036** Implement deep merge for objects
- [ ] **T037** Implement replace strategy for arrays and primitives
- [ ] **T038** [P] Write unit tests for merge behavior (objects, arrays, primitives, nested structures)
- [ ] **T039** [P] Write integration tests for multi-level merging (global + project)

### 2.7 Path Handling

- [ ] **T040** Implement config path resolution using `dirs` crate
- [ ] **T041** Implement project detection (search upward for `.claude/config.json`)
- [ ] **T042** [P] Write unit tests for path resolution on all platforms
- [ ] **T043** [P] Write integration tests for project detection (create temp directories)

**Checkpoint**: ✅ Foundation complete - all core infrastructure ready, can now implement user stories in parallel

---

## Phase 3: User Story 1 - Basic Configuration File Management (Priority: P1) 🎯 MVP

**Goal**: Users can view and edit Claude Code config files through CLI

**Independent Test**: Read `~/.claude/config.json`, modify value, verify file updated correctly

### 3.1 ConfigManager Implementation

- [ ] **T044** Implement `ConfigManager` struct in `crates/core/src/config/manager.rs`
- [ ] **T045** Implement `ConfigManager::new()` constructor
- [ ] **T046** Implement `ConfigManager::get_global_config()`
- [ ] **T047** [P] Implement `ConfigManager::get_project_config(path)`
- [ ] **T048** [P] Implement `ConfigManager::get_merged_config(project_path)`
- [ ] **T049** Write unit tests for ConfigManager (mock filesystem)
- [ ] **T050** Write integration tests with real config files

### 3.2 CLI: config get Command

- [ ] **T051** Define CLI arguments in `crates/cli/src/commands/config.rs`
  - `get [key]` subcommand
  - `--project <path>` global flag
  - `--output json|table` format flag
- [ ] **T052** Implement `config_get()` function
- [ ] **T053** [P] Implement table output formatting in `crates/cli/src/output/table.rs`
- [ ] **T054** [P] Implement JSON output formatting in `crates/cli/src/output/json.rs`
- [ ] **T055** Write unit tests for output formatters
- [ ] **T056** Write integration tests for `config get` command

### 3.3 CLI: config set Command

- [ ] **T057** Define `set <key> <value>` subcommand
- [ ] **T058** Implement key path parsing (e.g., "mcpServers.npx.enabled")
- [ ] **T059** Implement value setting (JSON parsing for objects/arrays)
- [ ] **T060** Call ConfigManager to write config with backup
- [ ] **T061** [P] Write unit tests for key path parsing
- [ ] **T062** Write integration tests for `config set` command
- [ ] **T063** [P] Write test for backup creation on set

### 3.4 Error Messages

- [ ] **T064** Improve error messages for common scenarios:
  - File not found (suggest creating config)
  - Invalid JSON (show line number)
  - Validation failed (explain what's wrong)
  - Permission denied (suggest fix)
- [ ] **T065** Write integration tests verifying error message quality

**Checkpoint**: ✅ US1 complete - Users can view and modify configs via CLI

---

## Phase 4: User Story 2 - Multi-Level Configuration Hierarchy (Priority: P1)

**Goal**: Global + project configs with merge and diff

**Independent Test**: Create global + project configs, verify merge and diff work correctly

### 4.1 ConfigManager Enhancements

- [ ] **T066** Implement `ConfigManager::update_global_config()`
- [ ] **T067** [P] Implement `ConfigManager::update_project_config(path, config)`
- [ ] **T068** Implement `ConfigManager::diff_configs(project_path)`
- [ ] **T069** [P] Implement `SourceMap` to track value origins
- [ ] **T070** Write unit tests for config diff
- [ ] **T071** Write integration tests for merge scenarios

### 4.2 CLI: config diff Command

- [ ] **T072** Define `diff [project-path]` subcommand
- [ ] **T073** Implement diff visualization (show additions, removals, modifications)
- [ ] **T074** [P] Add color coding for diff output (green = added, red = removed, yellow = modified)
- [ ] **T075** Write integration tests for `config diff` command

### 4.3 Project Detection

- [ ] **T076** Implement automatic project detection when `--project` flag not provided
- [ ] **T077** Add caching for detected project path (session-level)
- [ ] **T078** Write integration tests for auto-detection
- [ ] **T079** [P] Write edge case tests (no .claude found, nested .claude directories)

**Checkpoint**: ✅ US2 complete - Multi-level config with merge and diff working

---

## Phase 5: User Story 3 - MCP Servers Management (Priority: P1)

**Goal**: Manage MCP Servers (list, enable/disable, add, remove)

**Independent Test**: List servers, enable/disable, verify config file updates

### 5.1 McpManager Implementation

- [ ] **T080** Create `McpManager` in `crates/core/src/mcp/manager.rs`
- [ ] **T081** Implement `McpManager::list_servers(scope, project_path)`
- [ ] **T082** Implement `McpManager::enable_server(name, scope, project_path)`
- [ ] **T083** [P] Implement `McpManager::disable_server(name, scope, project_path)`
- [ ] **T084** Implement `McpManager::add_server(server, scope, project_path)`
- [ ] **T085** [P] Implement `McpManager::remove_server(name, scope, project_path)`
- [ ] **T086** Write unit tests for all McpManager methods
- [ ] **T087** Write integration tests with real config files

### 5.2 CLI: mcp Commands

- [ ] **T088** Define `mcp` subcommand group in `crates/cli/src/commands/mcp.rs`
- [ ] **T089** Implement `mcp list [--scope]` command
- [ ] **T090** Implement `mcp enable <name> [--scope]` command
- [ ] **T091** [P] Implement `mcp disable <name> [--scope]` command
- [ ] **T092** Implement `mcp add <name> [--args] [--env]` command
- [ ] **T093** [P] Implement `mcp remove <name> [--scope]` command
- [ ] **T094** Implement `mcp show <name>` command (display details)
- [ ] **T095** Write integration tests for all mcp commands

**Checkpoint**: ✅ US3 complete - Full MCP server management via CLI

---

## Phase 6: User Story 4 - Configuration Validation and Safety (Priority: P1)

**Goal**: Validate configs, create backups, atomic writes

**Independent Test**: Attempt invalid operations, verify they're rejected with clear errors

### 6.1 Validation Integration

- [ ] **T096** Integrate validation into `write_config_with_backup()`
- [ ] **T097** [P] Add pre-write validation for all config modifications
- [ ] **T098** Implement validation error messages with actionable suggestions
- [ ] **T099** Write integration tests for validation scenarios

### 6.2 Backup System Integration

- [ ] **T100** Ensure all write operations create backups automatically
- [ ] **T101** [P] Implement backup cleanup (keep last 10)
- [ ] **T102** Add `ccm history list` command to show backups
- [ ] **T103** [P] Add `ccm history restore <backup-file>` command
- [ ] **T104** Write integration tests for backup/restore workflow

### 6.3 Atomic Write Verification

- [ ] **T105** Write integration test simulating crash during write
- [ ] **T106** [P] Verify original file intact after failed write
- [ ] **T107** Test atomic rename on different filesystems (error handling)

**Checkpoint**: ✅ US4 complete - All config operations safe and validated

---

## Phase 7: User Story 5 - Project Discovery and Scanning (Priority: P2)

**Goal**: Scan filesystem for projects with .claude directories

**Independent Test**: Scan directory tree, verify all projects found

### 7.1 ProjectScanner Implementation

- [ ] **T108** Create `ProjectScanner` in `crates/core/src/project/scanner.rs`
- [ ] **T109** Implement `scan_directory(path)` method
- [ ] **T110** Implement parallel directory traversal (rayon for parallelism)
- [ ] **T111** [P] Add filtering (depth limit, ignore patterns)
- [ ] **T112** Write unit tests for scanner logic
- [ ] **T113** Write integration tests with real directory structures

### 7.2 CLI: project Commands

- [ ] **T114** Implement `project scan [path]` command
- [ ] **T115** [P] Implement `project list` command
- [ ] **T116** Implement `project config <path>` command
- [ ] **T117** Add table output formatting for project list
- [ ] **T118** Write integration tests for project commands

**Checkpoint**: ✅ US5 complete - Project discovery and scanning working

---

## Phase 8: User Story 6 - Configuration Search and Query (Priority: P2)

**Goal**: Search config values across all levels

**Independent Test**: Set values at different levels, search finds all occurrences

### 8.1 Search Implementation

- [ ] **T119** Add `search_config(query, scope)` method to ConfigManager
- [ ] **T120** Implement recursive key search through config objects
- [ ] **T121** [P] Implement value search (filter by value)
- [ ] **T122** Return results with source (global/project) and path
- [ ] **T123** Write unit tests for search logic
- [ ] **T124** Write integration tests for search scenarios

### 8.2 CLI: search Command

- [ ] **T125** Implement `search <query> [--key] [--value]` command
- [ ] **T126** Format search results with source highlighting
- [ ] **T127** [P] Support regex patterns in search queries
- [ ] **T128** Write integration tests for search command

**Checkpoint**: ✅ US6 complete - Configuration search working

---

## Phase 9: User Story 7 - Configuration Import/Export (Priority: P2)

**Goal**: Export config to file, import from file

**Independent Test**: Export config, modify, import, verify changes applied

### 9.1 Import/Export Implementation

- [ ] **T129** Implement `export_config(path)` method
- [ ] **T130** [P] Implement `import_config(path)` with validation
- [ ] **T131** Add format detection (JSON/TOML) for import
- [ ] **T132** Write unit tests for import/export
- [ ] **T133** [P] Write integration tests with import/export cycle

### 9.2 CLI: import/export Commands

- [ ] **T134** Implement `config export <file>` command
- [ ] **T135** [P] Implement `config import <file>` command
- [ ] **T136** Add validation before import
- [ ] **T137** Write integration tests for import/export commands

**Checkpoint**: ✅ US7 complete - Import/export working

---

## Phase 10: User Story 8 - Configuration History and Rollback (Priority: P3)

**Goal**: View history, rollback to previous versions

**Independent Test**: Make changes, view history, rollback, verify old config restored

### 10.1 History Management

- [ ] **T138** Enhance `BackupManager` with metadata tracking
- [ ] **T139** [P] Add change log (what changed, when, why)
- [ ] **T140** Implement `list_history()` method with metadata
- [ ] **T141** Implement `restore_backup(backup_id)` method
- [ ] **T142** Write unit tests for history tracking
- [ ] **T143** Write integration tests for restore scenarios

### 10.2 CLI: history Commands

- [ ] **T144** Implement `history list` command (already partially done in T102)
- [ ] **T145** [P] Implement `history restore <backup-id>` command (already partially done in T103)
- [ ] **T146** Add human-readable timestamps to history list
- [ ] **T147** Write integration tests for history commands

**Checkpoint**: ✅ US8 complete - History and rollback working

---

## Phase 11: Cross-Platform Testing & Quality Assurance

**Goal**: Ensure tool works correctly on Windows, macOS, and Linux

### 11.1 Platform-Specific Tests

- [ ] **T148** Run full test suite on Windows (GitHub Actions)
- [ ] **T149** [P] Run full test suite on macOS (GitHub Actions)
- [ ] **T150** [P] Run full test suite on Linux (GitHub Actions)
- [ ] **T151** Fix any platform-specific failures

### 11.2 Performance Testing

- [ ] **T152** Benchmark CLI startup time (target: <100ms)
- [ ] **T153** [P] Benchmark config parsing (target: <10ms for <100KB files)
- [ ] **T154** Profile and optimize hot paths if needed
- [ ] **T155** Add performance tests to CI

### 11.3 Code Quality

- [ ] **T156** Ensure 100% of code is clippy-clean (no warnings)
- [ ] **T157** [P] Ensure 100% of code is rustfmt-compliant
- [ ] **T158** Run cargo-deny to check for security advisories
- [ ] **T159** Generate test coverage report (target: >90%)

**Checkpoint**: ✅ Cross-platform compatibility verified, performance targets met

---

## Phase 12: Documentation and Examples

**Goal**: Comprehensive documentation for users and contributors

### 12.1 User Documentation

- [ ] **T160** Complete README.md with:
  - Installation instructions
  - Quick start guide
  - Command reference
  - Examples
  - Troubleshooting
- [ ] **T161** [P] Create man pages for all commands (using help2man or similar)
- [ ] **T162** Add shell completion scripts (bash, zsh, fish)
- [ ] **T163** Create example configs in `examples/` directory

### 12.2 Developer Documentation

- [ ] **T164** Add rustdoc comments to all public APIs
- [ ] **T165** [P] Create CONTRIBUTING.md with:
  - Development setup
  - Testing guidelines
  - Code style guide
  - PR process
- [ ] **T166** Create ARCHITECTURE.md explaining:
  - Three-layer architecture
  - Data flow
  - Design decisions

**Checkpoint**: ✅ Documentation complete, project ready for release

---

## Phase 13: Release Preparation

**Goal**: Prepare for first release

### 13.1 Build and Distribution

- [ ] **T167** Set up release build script
- [ ] **T168** [P] Create GitHub Actions workflow for building releases
- [ ] **T169** Build binaries for:
  - Windows x86_64
  - macOS x86_64 and aarch64
  - Linux x86_64
- [ ] **T170** Test installation from binaries

### 13.2 Release Checklist

- [ ] **T171** Update version numbers in Cargo.toml
- [ ] **T172** [P] Create CHANGELOG.md with release notes
- [ ] **T173** Create Git tag for release
- [ ] **T174** Create GitHub Release with binaries
- [ ] **T175** Publish to crates.io (optional)

**Checkpoint**: ✅ Release complete, tool available for users

---

## Task Statistics

- **Total Tasks**: 175
- **Foundational (Blocking)**: 43 tasks (T001-T043)
- **User Story 1 (P1)**: 22 tasks (T044-T065)
- **User Story 2 (P1)**: 14 tasks (T066-T079)
- **User Story 3 (P1)**: 18 tasks (T080-T095)
- **User Story 4 (P1)**: 12 tasks (T096-T107)
- **User Story 5 (P2)**: 11 tasks (T108-T118)
- **User Story 6 (P2)**: 10 tasks (T119-T128)
- **User Story 7 (P2)**: 9 tasks (T129-T137)
- **User Story 8 (P3)**: 10 tasks (T138-T147)
- **QA & Documentation**: 28 tasks (T148-T175)
- **Parallelizable**: ~60 tasks marked with [P]

## Estimated Effort

Based on task complexity:
- **Foundation**: 2-3 weeks (critical path, no parallelism)
- **US1-US4 (P1)**: 3-4 weeks (can parallelize some work)
- **US5-US7 (P2)**: 2-3 weeks
- **US8 (P3)**: 1 week
- **QA & Docs**: 1-2 weeks

**Total**: 9-13 weeks for initial implementation

## Implementation Order

**Recommended sequence** (respecting dependencies):

1. ✅ Phase 1-2: Foundation (MUST complete first)
2. ✅ Phase 3: US1 - Basic config management (MVP core)
3. ✅ Phase 4: US2 - Multi-level hierarchy (MVP complete)
4. ✅ Phase 5: US3 - MCP management (feature complete)
5. ✅ Phase 6: US4 - Validation and safety (quality gate)
6. ⏸️ Phase 7-8: US5-US6 - Additional features (can defer if needed)
7. ⏸️ Phase 9-10: US7-US8 - Nice-to-have features (can defer)
8. ✅ Phase 11-13: QA, Docs, Release (final push)

**MVP Definition**: Phases 1-6 complete (US1-US4)
**Feature Complete**: Phases 1-10 complete (US1-US8)
**Production Ready**: Phases 1-13 complete

## Next Steps

1. Start with Phase 1 (Project Setup)
2. Move to Phase 2 (Foundation) - blocking prerequisite
3. Implement US1-US4 in priority order (P1 features)
4. QA and testing throughout (TDD per Constitution)
5. Consider parallelizing [P] tasks where possible

**Constitution Compliance**: All tasks follow Constitution principles (TDD, safety, cross-platform, etc.)
