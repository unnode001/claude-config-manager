# Claude Code Configuration Format Specification

**Version**: 1.0
**Last Updated**: 2025-01-19
**Source**: Reverse-engineered from Claude Code documentation and examples

## Overview

This document specifies the JSON configuration format used by Claude Code. This is an **unofficial** specification based on observation and documentation, as Claude Code does not publish a formal schema.

## File Location

### Global Configuration
- **Windows**: `%APPDATA%\claude\config.json`
- **macOS**: `~/.claude/config.json`
- **Linux**: `~/.claude/config.json`

### Project Configuration
- **Path**: `<project-root>/.claude/config.json`

## Format Specification

### Root Object

The configuration file is a JSON object with the following top-level keys:

```json
{
  "mcpServers": { /* object */ },
  "allowedPaths": [ /* array */ ],
  "skills": { /* object */ },
  "customInstructions": [ /* array */ ]
}
```

### mcpServers (Object, Optional)

Map of MCP (Model Context Protocol) server configurations.

**Structure**:
```json
{
  "mcpServers": {
    "<server-name>": {
      "enabled": true,
      "args": ["--arg1", "arg2"],
      "env": {
        "KEY": "value"
      }
    }
  }
}
```

**Fields**:
- `<server-name>`: Unique identifier for the server (string)
  - **enabled** (boolean, required): Whether the server is enabled
  - **args** (array of strings, optional): Command-line arguments
  - **env** (object of string→string, optional): Environment variables

**Example**:
```json
{
  "mcpServers": {
    "npx": {
      "enabled": true,
      "args": ["--registry", "https://registry.npmjs.org"]
    },
    "filesystem": {
      "enabled": true,
      "args": []
    },
    "custom-server": {
      "enabled": false,
      "args": ["--port", "8080"],
      "env": {
        "API_KEY": "sk-..."
      }
    }
  }
}
```

**Validation Rules**:
- `enabled` must be boolean
- `args` must be array of strings if present
- `env` must be object with string values if present
- Server names must be valid JSON object keys (strings)

### allowedPaths (Array, Optional)

List of filesystem paths that Claude Code is allowed to access.

**Structure**:
```json
{
  "allowedPaths": [
    "~/projects",
    "/usr/local",
    "C:\\Projects"
  ]
}
```

**Fields**:
- Array of strings representing filesystem paths
- Paths can use `~` for home directory
- Paths use platform-specific separators (`/` or `\\`)

**Example**:
```json
{
  "allowedPaths": [
    "~/code",
    "~/projects",
    "/usr/local/share"
  ]
}
```

**Validation Rules**:
- Must be array if present
- All elements must be strings
- Paths should be valid (syntactically), but not validated for existence

### skills (Object, Optional)

Map of Claude Code skill configurations.

**Structure**:
```json
{
  "skills": {
    "<skill-name>": {
      "enabled": true,
      "parameters": {
        "param1": "value1",
        "param2": 42
      }
    }
  }
}
```

**Fields**:
- `<skill-name>`: Unique identifier for the skill (string)
  - **enabled** (boolean, required): Whether the skill is enabled
  - **parameters** (object, optional): Skill-specific parameters
    - Values can be strings, numbers, booleans, arrays, or objects

**Example**:
```json
{
  "skills": {
    "code-review": {
      "enabled": true,
      "parameters": {
        "strictness": "high",
        "maxLines": 500
      }
    },
    "documentation": {
      "enabled": false,
      "parameters": {}
    }
  }
}
```

**Validation Rules**:
- `enabled` must be boolean
- `parameters` values can be any JSON type
- Parameter names and values are not validated (skill-specific)

### customInstructions (Array, Optional)

List of custom instructions for Claude Code to follow.

**Structure**:
```json
{
  "customInstructions": [
    "Be concise in your responses",
    "Always provide code examples",
    "Prefer functional programming patterns"
  ]
}
```

**Fields**:
- Array of instruction strings

**Example**:
```json
{
  "customInstructions": [
    "Use Rust 2021 edition",
    "Follow the API guidelines",
    "Include documentation examples"
  ]
}
```

**Validation Rules**:
- Must be array if present
- All elements must be strings
- Empty array is valid

## Unknown Fields

**Handling**: The tool should preserve unknown fields without validation.

**Rationale**: Future versions of Claude Code may add new fields. Preserving unknown fields ensures forward compatibility.

**Example**:
```json
{
  "mcpServers": { "npx": { "enabled": true } },
  "futureFeature": { /* unknown, preserve as-is */ }
}
```

## Type Specifications

### JSON Schema

While Claude Code doesn't publish a schema, the following represents the validation rules:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "mcpServers": {
      "type": "object",
      "patternProperties": {
        ".*": {
          "type": "object",
          "required": ["enabled"],
          "properties": {
            "enabled": { "type": "boolean" },
            "args": {
              "type": "array",
              "items": { "type": "string" }
            },
            "env": {
              "type": "object",
              "patternProperties": {
                ".*": { "type": "string" }
              }
            }
          },
          "additionalProperties": false
        }
      }
    },
    "allowedPaths": {
      "type": "array",
      "items": { "type": "string" }
    },
    "skills": {
      "type": "object",
      "patternProperties": {
        ".*": {
          "type": "object",
          "required": ["enabled"],
          "properties": {
            "enabled": { "type": "boolean" },
            "parameters": { "type": "object" }
          },
          "additionalProperties": false
        }
      }
    },
    "customInstructions": {
      "type": "array",
      "items": { "type": "string" }
    }
  },
  "additionalProperties": true
}
```

## Validation Rules

### Required Fields
None. All top-level fields are optional.

### Type Checking
- `mcpServers.<name>.enabled`: **MUST** be boolean
- `mcpServers.<name>.args`: **MUST** be array of strings if present
- `mcpServers.<name>.env`: **MUST** be object with string values if present
- `allowedPaths`: **MUST** be array of strings if present
- `skills.<name>.enabled`: **MUST** be boolean
- `skills.<name>.parameters`: **MUST** be object if present
- `customInstructions`: **MUST** be array of strings if present

### Semantic Validation

**Not enforced** (Claude Code's responsibility):
- Whether server names are valid
- Whether paths exist
- Whether skill parameters are correct
- Whether custom instructions are valid

**Our tool's responsibility**:
- Type checking (above)
- JSON syntax validation
- Basic structural validation
- Preserve unknown fields

## Examples

### Minimal Configuration

```json
{
  "mcpServers": {
    "npx": {
      "enabled": true
    }
  }
}
```

### Typical Configuration

```json
{
  "mcpServers": {
    "npx": {
      "enabled": true,
      "args": ["--registry", "https://registry.npmjs.org"]
    },
    "filesystem": {
      "enabled": true,
      "args": []
    }
  },
  "allowedPaths": [
    "~/projects",
    "~/code"
  ],
  "skills": {
    "code-review": {
      "enabled": true,
      "parameters": {
        "strictness": "medium"
      }
    }
  },
  "customInstructions": [
    "Prefer clear, concise explanations",
    "Include code examples"
  ]
}
```

### Empty Configuration

```json
{}
```

**Note**: Empty config is valid. Claude Code will use defaults.

## Versioning and Migrations

### Current Version
- **Format Version**: 1.0 (as of 2025-01-19)
- **Claude Code Version**: Compatible with Claude Code 1.x

### Forward Compatibility
- **Unknown fields**: Preserve without modification
- **New optional fields**: Ignore when reading, preserve when writing
- **Removed fields**: Preserve when writing (don't delete)

### Migration Strategy
When Claude Code's format changes:
1. Detect version via schema version field (if added) or heuristic
2. Apply migration transformations
3. Validate migrated config
4. Create backup before migration
5. Log migration details

## Implementation Notes

### Parsing
- Use `serde_json` for parsing (Rust)
- Validate types after parsing
- Preserve unknown fields using `HashMap<String, Value>`

### Merging
- **Objects**: Deep merge (recursive)
- **Arrays**: Replace (no element-wise merge)
- **Primitives**: Replace with higher-priority value

### Writing
- Serialize with `serde_json::to_string_pretty`
- Use 2-space indentation (matches Claude Code)
- Ensure trailing newline
- Preserve unknown fields
- Validate before writing

### Atomic Write
1. Create backup
2. Write to temp file
3. Validate temp file
4. Atomic rename (temp → config)

## Testing

### Test Cases

**Valid Configs**:
- Minimal config (single field)
- Full config (all fields)
- Empty config
- Config with unknown fields

**Invalid Configs**:
- Malformed JSON
- Wrong types (e.g., `enabled: "true"` instead of `enabled: true`)
- Missing required fields (none required, but test validation)

**Edge Cases**:
- Very large config (>1MB)
- Unicode characters in paths
- Special characters in strings
- Nested objects in parameters

## References

- Claude Code Documentation: https://docs.anthropic.com/claude-code
- MCP Specification: https://modelcontextprotocol.io/
- JSON Schema: https://json-schema.org/

## Changelog

### 2025-01-19
- Initial version (1.0)
- Reverse-engineered from Claude Code 1.x
