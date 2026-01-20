//! Key path parsing and manipulation
//!
//! Supports dot-notation key paths like "mcpServers.npx.enabled"

use anyhow::Result;
use claude_config_manager_core::ClaudeConfig;
use serde_json::Value;

/// Parse and set a value using a key path
///
/// # Arguments
/// * `config` - The configuration to modify
/// * `key_path` - Dot-separated key path (e.g., "mcpServers.npx.enabled")
/// * `value` - The value to set (as JSON string)
pub fn set_value_by_path(config: &mut ClaudeConfig, key_path: &str, value: &str) -> Result<()> {
    let keys: Vec<&str> = key_path.split('.').collect();

    if keys.is_empty() {
        anyhow::bail!("Key path cannot be empty");
    }

    // Parse the value as JSON
    let parsed_value = parse_value(value)?;

    // Special handling for known top-level keys
    match keys[0] {
        "mcpServers" => set_mcp_server_value(config, &keys[1..], parsed_value)?,
        "allowedPaths" => set_allowed_paths_value(config, &keys[1..], parsed_value)?,
        "skills" => set_skill_value(config, &keys[1..], parsed_value)?,
        "customInstructions" => set_custom_instruction_value(config, &keys[1..], parsed_value)?,
        _ => {
            // Unknown field - add to unknown map
            set_unknown_value(config, &keys, parsed_value)?;
        }
    }

    Ok(())
}

/// Parse a value string as JSON
fn parse_value(value: &str) -> Result<Value> {
    // Try to parse as JSON first
    if let Ok(parsed) = serde_json::from_str::<Value>(value) {
        return Ok(parsed);
    }

    // If that fails, treat as a string
    Ok(Value::String(value.to_string()))
}

/// Set a value in the mcpServers section
fn set_mcp_server_value(config: &mut ClaudeConfig, keys: &[&str], value: Value) -> Result<()> {
    if keys.is_empty() {
        anyhow::bail!("MCP server name is required");
    }

    let server_name = keys[0];

    // Get or create the mcp_servers map
    let servers = config.mcp_servers.get_or_insert_with(Default::default);

    // Get or create the server
    let server = servers.entry(server_name.to_string()).or_insert_with(|| {
        claude_config_manager_core::McpServer::new(server_name, "", vec![])
    });

    // Set the specific field
    if keys.len() == 1 {
        // Setting the entire server - not supported in this simple version
        anyhow::bail!("Setting entire server object is not yet supported. Use 'enabled', 'command', or 'args'");
    }

    let field = keys[1];

    match field {
        "enabled" => {
            if let Some(bool_val) = value.as_bool() {
                server.enabled = bool_val;
            } else if let Some(string_val) = value.as_str() {
                server.enabled = string_val.eq_ignore_ascii_case("true") ||
                               string_val.eq_ignore_ascii_case("yes") ||
                               string_val == "1";
            } else {
                anyhow::bail!("'enabled' must be a boolean value");
            }
        }
        "command" => {
            if let Some(string_val) = value.as_str() {
                server.command = Some(string_val.to_string());
            } else {
                anyhow::bail!("'command' must be a string");
            }
        }
        "args" => {
            match value {
                Value::Array(arr) => {
                    server.args = arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
                Value::String(s) => {
                    // Split string by spaces
                    server.args = s.split_whitespace().map(|s| s.to_string()).collect();
                }
                _ => {
                    anyhow::bail!("'args' must be an array or a space-separated string");
                }
            }
        }
        _ => {
            anyhow::bail!("Unknown MCP server field: '{}'", field);
        }
    }

    Ok(())
}

/// Set a value in the allowedPaths section
fn set_allowed_paths_value(config: &mut ClaudeConfig, keys: &[&str], value: Value) -> Result<()> {
    if !keys.is_empty() {
        anyhow::bail!("Nested paths in allowedPaths are not supported");
    }

    match value {
        Value::Array(arr) => {
            config.allowed_paths = Some(
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            );
        }
        Value::String(s) => {
            config.allowed_paths = Some(vec![s]);
        }
        _ => {
            anyhow::bail!("allowedPaths must be an array or string");
        }
    }

    Ok(())
}

/// Set a value in the skills section
fn set_skill_value(config: &mut ClaudeConfig, keys: &[&str], value: Value) -> Result<()> {
    if keys.is_empty() {
        anyhow::bail!("Skill name is required");
    }

    let skill_name = keys[0];

    // Get or create the skills map
    let skills = config.skills.get_or_insert_with(Default::default);

    // Get or create the skill
    let skill = skills.entry(skill_name.to_string()).or_insert_with(|| {
        claude_config_manager_core::Skill {
            name: skill_name.to_string(),
            enabled: true,
            parameters: None,
        }
    });

    // Set the specific field
    if keys.len() == 1 {
        // Setting the entire skill
        anyhow::bail!("Setting entire skill object is not yet supported");
    }

    let field = keys[1];

    match field {
        "enabled" => {
            if let Some(bool_val) = value.as_bool() {
                skill.enabled = bool_val;
            } else if let Some(string_val) = value.as_str() {
                skill.enabled = string_val.eq_ignore_ascii_case("true") ||
                               string_val.eq_ignore_ascii_case("yes") ||
                               string_val == "1";
            } else {
                anyhow::bail!("'enabled' must be a boolean value");
            }
        }
        "parameters" => {
            skill.parameters = Some(value);
        }
        _ => {
            anyhow::bail!("Unknown skill field: '{}'", field);
        }
    }

    Ok(())
}

/// Set a value in the customInstructions section
fn set_custom_instruction_value(config: &mut ClaudeConfig, keys: &[&str], value: Value) -> Result<()> {
    if !keys.is_empty() {
        anyhow::bail!("Nested paths in customInstructions are not supported");
    }

    let instructions = config.custom_instructions.get_or_insert_with(Vec::new);

    match value {
        Value::Array(arr) => {
            *instructions = arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
        Value::String(s) => {
            instructions.push(s);
        }
        _ => {
            anyhow::bail!("customInstructions must be an array or string");
        }
    }

    Ok(())
}

/// Set a value in the unknown fields map
fn set_unknown_value(config: &mut ClaudeConfig, keys: &[&str], value: Value) -> Result<()> {
    if keys.is_empty() {
        anyhow::bail!("Key path cannot be empty");
    }

    // For unknown fields, we only support top-level setting for now
    if keys.len() > 1 {
        anyhow::bail!("Nested paths for unknown fields are not supported");
    }

    config.unknown.insert(keys[0].to_string(), value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use claude_config_manager_core::McpServer;

    #[test]
    fn test_parse_value_json() {
        let result = parse_value("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_parse_value_string() {
        let result = parse_value("hello");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::String("hello".to_string()));
    }

    #[test]
    fn test_parse_value_json_array() {
        let result = parse_value("[\"a\", \"b\"]");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.is_array());
    }

    #[test]
    fn test_set_mcp_server_enabled() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "mcpServers.npx.enabled", "true").unwrap();

        assert!(config.mcp_servers.is_some());
        let servers = config.mcp_servers.unwrap();
        assert!(servers.contains_key("npx"));
        let server = servers.get("npx").unwrap();
        assert_eq!(server.enabled, true);
    }

    #[test]
    fn test_set_allowed_paths_string() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "allowedPaths", "~/projects").unwrap();

        assert!(config.allowed_paths.is_some());
        let paths = config.allowed_paths.unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "~/projects");
    }

    #[test]
    fn test_set_allowed_paths_array() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "allowedPaths", "[\"~/projects\", \"~/work\"]").unwrap();

        assert!(config.allowed_paths.is_some());
        let paths = config.allowed_paths.unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_set_custom_instructions() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "customInstructions", "Be concise").unwrap();

        assert!(config.custom_instructions.is_some());
        let instructions = config.custom_instructions.unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0], "Be concise");
    }

    #[test]
    fn test_set_skill_enabled() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "skills.code-review.enabled", "false").unwrap();

        assert!(config.skills.is_some());
        let skills = config.skills.unwrap();
        assert!(skills.contains_key("code-review"));
        let skill = skills.get("code-review").unwrap();
        assert_eq!(skill.enabled, false);
    }

    #[test]
    fn test_set_unknown_field() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "myField", "myValue").unwrap();

        assert!(config.unknown.contains_key("myField"));
        assert_eq!(config.unknown.get("myField"), Some(&Value::String("myValue".to_string())));
    }

    #[test]
    fn test_set_mcp_server_command() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "mcpServers.npx.command", "npx").unwrap();

        assert!(config.mcp_servers.is_some());
        let servers = config.mcp_servers.unwrap();
        let server = servers.get("npx").unwrap();
        assert_eq!(server.command, Some("npx".to_string()));
    }

    #[test]
    fn test_set_mcp_server_args_array() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "mcpServers.npx.args", "[\"-y\", \"--registry\", \"https://registry.npmjs.org\"]").unwrap();

        assert!(config.mcp_servers.is_some());
        let servers = config.mcp_servers.unwrap();
        let server = servers.get("npx").unwrap();
        assert_eq!(server.args.len(), 3);
    }

    #[test]
    fn test_set_mcp_server_args_string() {
        let mut config = ClaudeConfig::new();
        set_value_by_path(&mut config, "mcpServers.npx.args", "-y --registry https://registry.npmjs.org").unwrap();

        assert!(config.mcp_servers.is_some());
        let servers = config.mcp_servers.unwrap();
        let server = servers.get("npx").unwrap();
        assert_eq!(server.args.len(), 3);
    }
}
