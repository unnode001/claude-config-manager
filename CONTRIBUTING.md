# Contributing to Claude Config Manager

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community

## Development Setup

### Prerequisites

- Rust 1.75+
- Cargo (included with Rust)
- Git

### Setup

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/claude-config-manager.git
cd claude-config-manager

# Add upstream remote
git remote add upstream https://github.com/original-org/claude-config-manager.git

# Install development tools
rustup component add rustfmt clippy
```

### Building

```bash
# Build all crates
cargo build --workspace

# Build release version
cargo build --release --workspace
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run specific test module
cargo test --test mcp_manager

# Run integration tests only
cargo test --test *_integration
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Check formatting without making changes
cargo fmt --all -- --check

# Run linter
cargo clippy --workspace

# Fix linter warnings
cargo clippy --fix --workspace --allow-dirty
```

## Making Changes

### Workflow

1. **Create a branch** for your changes
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes** following our coding standards

3. **Write tests** for your changes (TDD is encouraged)
   ```bash
   # Run tests related to your changes
   cargo test your_test_name
   ```

4. **Format code** and run linter
   ```bash
   cargo fmt --all
   cargo clippy --workspace
   ```

5. **Commit your changes** with a clear message
   ```bash
   git add .
   git commit -m "feat: add support for new feature"
   ```

6. **Push to your fork** and create a pull request

### Commit Message Format

Use conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add command for importing configurations
fix: resolve backup file naming collision
docs: update README with new command examples
test: add integration tests for search functionality
```

## Coding Standards

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for code formatting
- Address `clippy` warnings
- Write documentation comments (`///`) for public APIs
- Include examples in documentation

### Testing Guidelines

- Write tests before implementation (TDD)
- Unit tests in the same file as the code
- Integration tests in `tests/` directory
- Use descriptive test names
- Test both success and failure cases

Example:
```rust
#[test]
fn test_add_duplicate_server_fails() {
    // Arrange
    let manager = McpManager::new();
    let server = McpServer::new("test", "npx", vec![]);

    // Act
    manager.add_server(server.clone()).unwrap();
    let result = manager.add_server(server);

    // Assert
    assert!(result.is_err());
    assert!(matches!(result, Err(ConfigError::ValidationFailed(_, _))));
}
```

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use our custom `ConfigError` type
- Provide clear, actionable error messages
- Include context in error messages

Example:
```rust
// Good
Err(ConfigError::filesystem(
    "create backup directory",
    &backup_dir,
    e,
))

// Avoid
Err(ConfigError::IoError(e))
```

## Project Structure

```
claude-config-manager/
├── crates/
│   ├── core/              # Core library (no frontend dependencies)
│   │   ├── src/
│   │   │   ├── config/    # Configuration management
│   │   │   ├── mcp/       # MCP server management
│   │   │   ├── backup/    # Backup system
│   │   │   ├── search/    # Search functionality
│   │   │   ├── project/   # Project discovery
│   │   │   └── types.rs   # Shared types
│   │   └── tests/         # Integration tests
│   ├── cli/               # CLI application
│   │   └── src/
│   │       ├── commands/  # CLI command handlers
│   │       ├── key_path/  # Key path parsing
│   │       └── output/    # Output formatting
│   └── tauri/             # GUI (planned)
├── docs/                  # Documentation
├── specs/                 # Feature specifications
└── examples/              # Usage examples
```

## Adding Features

### 1. Update Specification

For significant features, update the relevant spec file in `specs/001-initial-implementation/`.

### 2. Implement in Core Library

Add business logic to `crates/core/src/` following the module structure.

### 3. Add CLI Command (if applicable)

Add command handler to `crates/cli/src/commands/`.

### 4. Write Tests

- Unit tests in the same file
- Integration tests in `crates/core/tests/`
- CLI tests in `crates/cli/tests/`

### 5. Update Documentation

- Update README.md if user-facing
- Add rustdoc comments to public APIs
- Update CHANGELOG.md

## Pull Request Process

### Before Submitting

- [ ] All tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation is updated
- [ ] Commit messages follow the format

### Submitting

1. Push your branch to your fork
2. Create a pull request on GitHub
3. Fill in the PR template
4. Link related issues

### PR Review

- Address review feedback
- Keep the PR focused and small
- Be responsive to comments

## Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and ideas
- **Specs**: See `specs/001-initial-implementation/` for design docs

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
