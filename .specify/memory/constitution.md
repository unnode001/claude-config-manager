<!--
Sync Impact Report:
- Version change: (initial) → 1.0.0
- Added sections: All sections (initial constitution)
- Templates updated:
  ✅ .specify/templates/plan-template.md (reviewed, no changes needed)
  ✅ .specify/templates/spec-template.md (reviewed, no changes needed)
  ✅ .specify/templates/tasks-template.md (reviewed, no changes needed)
- Follow-up TODOs: None
-->

# Claude Config Manager Constitution

## Core Principles

### I. Core Library First Architecture

Every feature MUST start as a standalone library in the `core` crate. The Core Library MUST be:

- **Self-contained**: All business logic resides in core, independent of any frontend
- **Independently testable**: Core MUST have full test coverage without requiring CLI or GUI
- **Well-documented**: Public APIs MUST have clear documentation and examples
- **Frontend-agnostic**: Core MUST NOT depend on CLI, Tauri, or any specific frontend framework

**Rationale**: This architecture enables multiple frontends (CLI, GUI, future APIs) to share the same battle-tested core logic, preventing code duplication and ensuring consistency across interfaces.

### II. Separation of Concerns

The project MUST maintain strict separation between three layers:

1. **Core Library** (`crates/core`): Business logic, config management, file operations
2. **CLI Frontend** (`crates/cli`): Command-line interface, argument parsing, text output
3. **GUI Frontend** (`crates/tauri`): Graphical interface, Tauri commands, UI state

**Rationale**: Clear boundaries enable parallel development, independent testing, and easier maintenance. Frontends are thin adapters that wrap Core functionality.

### III. Safety and Reliability

Configuration management operations MUST prioritize data safety:

- **Backup before modification**: All config file writes MUST create backups
- **Validation first**: Validate configs before writing (JSON schema, type checking)
- **Atomic operations**: Use write-then-rename pattern to prevent corruption
- **Clear error messages**: Errors MUST explain what went wrong and how to fix it
- **Never lose user data**: When in doubt, refuse operation rather than risk corruption

**Rationale**: Configuration files are critical user data. Corruption can break tools and lose important settings. Safety is non-negotiable.

### IV. Test-Driven Development (NON-NEGOTIABLE)

TDD is MANDATORY for all Core Library code:

1. Write test BEFORE writing implementation code
2. Tests MUST fail initially (Red)
3. Write minimal code to pass (Green)
4. Refactor for clarity (Refactor)
5. Repeat for each feature

**Test Requirements**:
- Unit tests: All public functions MUST have tests
- Integration tests: Config file operations MUST have integration tests
- Error cases: Test both success and failure paths
- Edge cases: Empty files, malformed JSON, missing directories

**Rationale**: TDD ensures code correctness, serves as living documentation, and prevents regressions. Core Library complexity demands rigorous testing.

### V. Configuration Hierarchy Management

The system MUST support multi-level configuration with clear precedence:

1. **System-level** (`~/.claude/config.json`): Global defaults
2. **Project-level** (`<project>/.claude/config.json`): Project-specific overrides
3. **Session-level** (optional): Temporary overrides

**Merge Rules**:
- Project config overrides System config
- Conflicts MUST be resolved with clear precedence: Project > System
- Merge MUST be traceable (users can see which level each value came from)
- Merge MUST be predictable and deterministic

**Rationale**: Different projects need different configurations. Clear hierarchy enables both global defaults and project-specific customization without confusion.

### VI. Extensibility and Plugin Support

The architecture MUST support future extensibility:

- **MCP Servers**: Modular, add/remove servers without code changes
- **Skills**: Pluggable skill system with custom skill support
- **Config formats**: Extensible parser for JSON/TOML/YAML
- **Export formats**: Config export to multiple formats (JSON, YAML, TOML)

**Rationale**: The CLI tools ecosystem evolves rapidly. The system must accommodate new tools, formats, and workflows without requiring rewrites.

### VII. Performance and Efficiency

Operations MUST be efficient for common use cases:

- **Lazy loading**: Load configs only when needed
- **Caching**: Cache parsed configs (invalidate on file change)
- **Incremental updates**: Update only what changed, not entire configs
- **Fast startup**: CLI MUST start quickly (<100ms for simple commands)

**Rationale**: Developers use this tool frequently. Slow tools disrupt workflow and reduce adoption.

### VIII. Cross-Platform Compatibility

The tool MUST work consistently across platforms:

- **Primary**: Windows, macOS, Linux
- **Config paths**: Use platform-appropriate directories (dirs crate)
- **Path handling**: Use path correctly for cross-platform compatibility
- **Line endings**: Handle CRLF/LF differences transparently

**Rationale**: Developers use different operating systems. The tool must work everywhere Claude Code runs.

## Additional Constraints

### Security Considerations

- **No sensitive data in logs**: Never log API keys, tokens, or passwords
- **File permissions**: Respect file permissions, don't expose protected configs
- **Input validation**: Validate all user inputs to prevent injection attacks
- **Safe defaults**: Default settings must be secure

### Technology Stack Requirements

- **Language**: Rust (Core Library, CLI, Tauri backend)
- **GUI Framework**: Tauri 2.x
- **Frontend**: React + TypeScript (Tauri frontend)
- **Config Parsing**: serde + serde_json + serde_toml
- **CLI**: clap
- **Testing**: built-in Rust test framework + rstest for parameterized tests
- **Path handling**: dirs, camino (for better cross-platform paths)

### Code Quality Standards

- **Linting**: Use clippy with strict warnings
- **Formatting**: Use rustfmt with default settings
- **Documentation**: Public APIs MUST have rustdoc comments
- **Error handling**: Use thiserror and anyhow appropriately
- **Logging**: Use tracing for structured logging

### Performance Standards

- **CLI startup**: <100ms for simple commands
- **Config parsing**: <10ms for typical config files (<100KB)
- **GUI responsiveness**: UI must remain responsive during long operations
- **Memory usage**: Efficient memory usage, no unnecessary allocations

## Development Workflow

### Feature Development Process

1. **Specification**: Use `/speckit.specify` to define feature requirements
2. **Clarification**: Use `/speckit.clarify` to resolve ambiguities
3. **Planning**: Use `/speckit.plan` to create technical design
4. **Tasks**: Use `/speckit.tasks` to break down implementation
5. **Implementation**: Use `/speckit.implement` following TDD
6. **Validation**: Manual testing and automated test verification

### Code Review Requirements

- All code changes MUST follow TDD principles
- All new code MUST have corresponding tests
- Integration tests MUST cover config file operations
- Error cases MUST be tested explicitly
- Documentation MUST be updated for API changes

### Quality Gates

- **Tests pass**: All tests MUST pass before merging
- **Clippy clean**: No clippy warnings allowed
- **Formatted**: Code must be rustfmt-compliant
- **Documented**: Public APIs must have rustdoc comments
- **Manual testing**: Config operations must be manually tested on real files

## Governance

### Amendment Procedure

The Constitution is the supreme governing document for this project. All development decisions MUST align with these principles.

**To amend the Constitution**:
1. Document the proposed change with rationale
2. Update version according to semantic versioning
3. Review impact on existing code and templates
4. Update dependent artifacts if needed
5. Record amendment in Sync Impact Report

**Version Policy**:
- MAJOR: Backward-incompatible changes (principle removal, redefinition)
- MINOR: New principles or major expansions
- PATCH: Clarifications, wording improvements, non-semantic changes

### Compliance Verification

- **Before implementation**: Check plan aligns with Constitution
- **During implementation**: Follow TDD and testing discipline
- **After implementation**: Verify tests pass and principles are upheld
- **Code review**: Ensure compliance with all principles

**Complexity Justification**: Any deviation from these principles MUST be explicitly justified and documented.

### Runtime Development Guidance

For implementation decisions not covered here, refer to:
- Rust API guidelines: https://rust-lang.github.io/api-guidelines/
- Tauri best practices: https://tauri.app/v2/guides/
- Claude Code documentation: https://docs.anthropic.com/claude-code

---

**Version**: 1.0.0 | **Ratified**: 2025-01-19 | **Last Amended**: 2025-01-19
