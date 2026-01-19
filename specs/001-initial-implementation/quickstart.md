# Quick Start Guide: Claude Config Manager

**Feature**: 001-initial-implementation
**Last Updated**: 2025-01-19

## Installation

### From Source

```bash
# Clone repository
git clone https://github.com/your-org/claude-config-manager.git
cd claude-config-manager

# Build and install
cargo install --path crates/cli

# Verify installation
ccm --version
```

### From Pre-built Binaries (Future)

Download from GitHub Releases:
- Windows: `ccm-windows-x86_64.exe`
- macOS: `ccm-macos-x86_64` or `ccm-macos-aarch64`
- Linux: `ccm-linux-x86_64`

## Basic Usage

### View Configuration

```bash
# View entire global config
ccm config get

# View specific key
ccm config get mcpServers.npx.enabled

# View project config (auto-detected)
ccm config get

# View specific project config
ccm config get --project ~/code/my-project
```

### Modify Configuration

```bash
# Set a value
ccm config set mcpServers.npx.enabled false

# Set multiple values
ccm config set allowedPaths '["~/projects", "~/work"]'

# Modify project-specific config
ccm config set mcpServers.npx.enabled true --project ~/code/my-project
```

### MCP Server Management

```bash
# List all MCP servers
ccm mcp list

# List globally enabled servers
ccm mcp list --scope global

# Enable a server globally
ccm mcp enable npx

# Enable a server for current project
ccm mcp enable npx --scope project

# Disable a server
ccm mcp disable npx

# Add a new server
ccm mcp add my-server --args "arg1" --args "arg2" --env "KEY=value"

# Remove a server
ccm mcp remove my-server

# Show server details
ccm mcp show npx
```

### Project Management

```bash
# Scan for projects with .claude directories
ccm project scan ~/code

# List all discovered projects
ccm project list

# Show project config
ccm project config ~/code/my-project

# Show diff between global and project
ccm config diff --project ~/code/my-project
```

### Configuration Import/Export

```bash
# Export current config
ccm config export my-config.json

# Export project config
ccm config export my-config.json --project ~/code/my-project

# Import config (validates first)
ccm config import my-config.json

# Import into project
ccm config import my-config.json --project ~/code/my-project
```

### Backup and Restore

```bash
# List backup history
ccm history list

# List backups for specific config
ccm history list --project ~/code/my-project

# Restore from backup
ccm history restore config.json.backup.20250119-143000

# Restore project backup
ccm history restore .claude/config.json.backup.20250119-150000 --project ~/code/my-project
```

## Output Formats

### Table Output (Default)

```bash
$ ccm mcp list
+------+---------+-------------+
| Name | Enabled | Args        |
+------+---------+-------------+
| npx  | true    | --registry  |
|      |         | https://...  |
+------+---------+-------------+
| fs   | true    |             |
+------+---------+-------------+
```

### JSON Output

```bash
$ ccm mcp list --output json
{
  "servers": [
    {
      "name": "npx",
      "enabled": true,
      "args": ["--registry", "https://..."]
    },
    {
      "name": "fs",
      "enabled": true,
      "args": []
    }
  ]
}
```

## Common Workflows

### Setup a New Project

```bash
# Navigate to project
cd ~/code/my-project

# Initialize project config
ccm config set allowedPaths '["~/code/my-project"]'

# Enable project-specific MCP server
ccm mcp enable custom-server --scope project

# Verify configuration
ccm config diff
```

### Switch Between Projects

```bash
# Work on project A
cd ~/code/project-a
ccm config get  # Auto-detects project-a config

# Work on project B
cd ~/code/project-b
ccm config get  # Auto-detects project-b config

# Or explicitly specify
ccm config get --project ~/code/project-c
```

### Troubleshooting

```bash
# Validate current config
ccm config validate

# Show where config values come from
ccm config get --verbose

# View recent backups
ccm history list

# Restore last known good config
ccm history restore $(ccm history list --json | jq -r '.[0].path')
```

## Configuration File Locations

### Global Config
- **Windows**: `%APPDATA%\claude\config.json`
- **macOS**: `~/.claude/config.json`
- **Linux**: `~/.claude/config.json`

### Project Config
- **Path**: `<project>/.claude/config.json`

### Backups
- **Naming**: `config.json.backup.YYYYMMDD-HHMMSS`
- **Location**: Same directory as config file

## Advanced Usage

### Custom Output Format

```bash
# Use jq for custom JSON processing
ccm config get --output json | jq '.mcpServers | keys[]'

# Use table output for human-readable format
ccm config get --output table
```

### Batch Operations

```bash
# Disable all MCP servers
for server in $(ccm mcp list --json | jq -r '.[].name'); do
    ccm mcp disable "$server"
done

# Set multiple values from JSON file
cat values.json | jq -r 'to_entries | .[] | "\(.key) \(.value)"' | \
    while read key value; do
        ccm config set "$key" "$value"
    done
```

### Shell Completion

```bash
# Generate completion script
ccm completion bash

# Enable for current session
source <(ccm completion bash)

# Enable permanently (bash)
ccm completion bash > ~/.local/share/bash-completion/completions/ccm

# Enable permanently (zsh)
ccm completion zsh > ~/.zfunc/_ccm
```

## Help and Documentation

```bash
# General help
ccm --help

# Command-specific help
ccm config --help
ccm mcp --help
ccm project --help

# Version info
ccm --version
```

## Examples

### Example 1: Quick Configuration Check

```bash
# View current config
$ ccm config get

# See what's different from global
$ ccm config diff

Changes in project config:
  - mcpServers.npx.enabled: true (global) → false (project)
  + allowedPaths: ["~/projects"] (project only)
```

### Example 2: MCP Server Setup

```bash
# Add a new MCP server
$ ccm mcp add my-custom-server --args "--port" --args "8080"

# Enable it globally
$ ccm mcp enable my-custom-server --scope global

# Verify it's enabled
$ ccm mcp show my-custom-server
Name: my-custom-server
Enabled: true
Scope: global
Args: --port 8080
```

### Example 3: Project-Specific Override

```bash
# Global config has npx enabled
$ ccm mcp show npx
Enabled: true (global)

# Disable for this project only
$ ccm mcp disable npx --scope project

# Verify
$ ccm config diff
mcpServers.npx.enabled: true (global) → false (project)
```

## Troubleshooting

### "Config file not found"

**Cause**: No config file exists yet.

**Solution**:
```bash
# Tool will create default config on first write
ccm config set customInstructions '["Be helpful"]'
```

### "Permission denied"

**Cause**: No write permission to config directory.

**Solution**:
```bash
# Check permissions
ls -la ~/.claude/

# Fix permissions
chmod 755 ~/.claude/
```

### "Invalid JSON"

**Cause**: Config file is corrupted.

**Solution**:
```bash
# Restore from backup
ccm history list
ccm history restore <backup-file>
```

## Next Steps

- Read [data-model.md](./data-model.md) for API details
- Read [plan.md](./plan.md) for architecture details
- Explore `examples/` directory for code examples
- Check [README.md](../../README.md) for full documentation

## Getting Help

- GitHub Issues: https://github.com/your-org/claude-config-manager/issues
- Documentation: https://docs.example.com/claude-config-manager
- Community Discord: https://discord.gg/example
