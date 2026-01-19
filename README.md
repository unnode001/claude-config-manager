# Claude Config Manager

> A centralized configuration management tool for Claude Code and other CLI-based AI development tools.

**Status**: 📋 Planning Complete | 🔨 Implementation Ready

## Overview

Claude Config Manager is a Rust-based command-line tool that provides fine-grained management of configuration files for Claude Code and other AI development tools. It supports multi-level configuration hierarchies (global/project/session), MCP Servers management, Skills configuration, and comprehensive safety features.

## Features

### ✅ Planned for Phase 1 (Initial Implementation)

- **Multi-Level Configuration Management**
  - Global configurations (`~/.claude/config.json`)
  - Project-specific configurations (`<project>/.claude/config.json`)
  - Smart merging with clear precedence rules
  - Configuration diff visualization

- **MCP Servers Management**
  - List, enable, disable, add, remove MCP servers
  - Scope control (global vs project-level)
  - Environment variable configuration
  - Connection testing

- **Configuration Safety**
  - Automatic backup creation before modifications
  - JSON schema validation
  - Atomic write operations (no corruption)
  - Clear, actionable error messages

- **Project Discovery**
  - Scan filesystem for projects with `.claude` directories
  - Automatic project detection
  - Project listing and management

- **Configuration Search & Query**
  - Search across all configuration levels
  - Find where values are defined
  - Trace value origins

- **Import/Export**
  - Export configurations to JSON files
  - Import configurations with validation
  - Share configurations across machines

### 🔮 Future Phases

- **Phase 2**: Interactive CLI mode, advanced history/audit log
- **Phase 3**: Tauri-based GUI application
- **Phase 4**: Support for other CLI tools (Codex, Cursor, etc.)

## Architecture

The project follows a **three-layer architecture**:

```
┌─────────────────────────────────────────┐
│  Frontend Layer                         │
│  ┌──────────┐      ┌──────────┐        │
│  │   CLI    │      │   GUI    │        │
│  │ (Rust)   │      │ (Tauri)  │        │
│  └──────────┘      └──────────┘        │
├─────────────────────────────────────────┤
│  Core Library (Rust)                    │
│  • Config management                    │
│  • MCP server management                │
│  • Validation & merging                 │
│  • Backup & atomic writes               │
└─────────────────────────────────────────┘
```

**Key Principles**:
- ✅ Core Library First (frontend-agnostic)
- ✅ Separation of Concerns (clear boundaries)
- ✅ Safety & Reliability (never lose user data)
- ✅ Test-Driven Development (TDD mandatory)
- ✅ Cross-Platform (Windows, macOS, Linux)

## Installation

### From Source (Coming Soon)

```bash
cargo install --path crates/cli
```

### Pre-built Binaries (Coming Soon)

Download from [GitHub Releases](https://github.com/your-org/claude-config-manager/releases)

## Quick Start

```bash
# View current configuration
ccm config get

# Set a configuration value
ccm config set mcpServers.npx.enabled false

# List MCP servers
ccm mcp list

# Enable a server for current project
ccm mcp enable custom-server --scope project

# Scan for projects
ccm project scan ~/code

# Show diff between global and project
ccm config diff
```

See [quickstart.md](specs/001-initial-implementation/quickstart.md) for detailed usage examples.

## Documentation

### Planning Documents

- **[Constitution](.specify/memory/constitution.md)** - Project principles and governance
- **[Feature Specification](specs/001-initial-implementation/spec.md)** - Complete functional requirements
- **[Implementation Plan](specs/001-initial-implementation/plan.md)** - Technical architecture and design
- **[Data Model](specs/001-initial-implementation/data-model.md)** - Core data structures
- **[Quick Start Guide](specs/001-initial-implementation/quickstart.md)** - Usage examples and tutorials
- **[Task List](specs/001-initial-implementation/tasks.md)** - 175 implementation tasks

### External Contracts

- **[Claude Code Config Format](specs/001-initial-implementation/contracts/claude-config-spec.md)** - Configuration format specification

## Development

### Prerequisites

- Rust 1.75+
- Cargo (included with Rust)
- Git

### Setup

```bash
# Clone repository
git clone https://github.com/your-org/claude-config-manager.git
cd claude-config-manager

# Run tests
cargo test --workspace

# Run linter
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all
```

### Project Structure

```
claude-config-manager/
├── crates/
│   ├── core/          # Core library (business logic)
│   ├── cli/           # CLI application
│   └── tauri/         # GUI application (deferred)
├── specs/             # Feature specifications and plans
├── tests/             # Integration tests
└── examples/          # Usage examples
```

### Testing

- **Unit Tests**: `cargo test --workspace`
- **Integration Tests**: `cargo test --test integration`
- **Coverage**: `cargo llvm-cov --workspace`

## Contributing

Contributions are welcome! Please see:

1. **[CONTRIBUTING.md](CONTRIBUTING.md)** (to be created) - Development guidelines
2. **[Code of Conduct](.github/CODE_OF_CONDUCT.md)** - Community guidelines
3. **[Spec-Driven Development](.specify/templates/)** - How we work

## Roadmap

### Phase 1: Core Implementation (Current) ✅
- **Status**: Planning complete, ready for implementation
- **Timeline**: 9-13 weeks
- **MVP**: Basic config management + MCP servers + safety features
- **See**: [Task List](specs/001-initial-implementation/tasks.md)

### Phase 2: Enhanced Features 📋
- Interactive CLI mode
- Advanced history and audit logging
- Configuration templates
- Shell completion improvements

### Phase 3: GUI Application 📋
- Tauri-based desktop application
- Real-time config monitoring
- Visual diff editor
- Project dashboard

### Phase 4: Multi-Tool Support 📋
- Codex CLI configuration
- Cursor configuration
- Generic tool framework
- Plugin system

## Performance

Targets for Phase 1:

- **CLI Startup**: <100ms for simple commands ✅
- **Config Parsing**: <10ms for <100KB files ✅
- **MCP Server List**: <50ms ✅
- **Config Write**: <50ms (including backup) ✅

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Core language
- [Tauri](https://tauri.app/) - GUI framework
- [Spec Kit](https://github.com/github/spec-kit) - Spec-driven development methodology
- [Claude Code](https://docs.anthropic.com/claude-code) - Inspiration and target tool

## Contact

- **GitHub Issues**: https://github.com/your-org/claude-config-manager/issues
- **Discussions**: https://github.com/your-org/claude-config-manager/discussions

---

**Note**: This project is currently in the planning phase. Implementation will begin after this README is reviewed and approved.

**Ready to start?** See the [Task List](specs/001-initial-implementation/tasks.md) to begin implementation!
