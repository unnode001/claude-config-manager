//! Table output formatter
//!
//! Formats configuration as human-readable tables

use anyhow::Result;
use claude_config_manager_core::ClaudeConfig;
use serde_json::Value;

/// Format configuration as a human-readable table
///
/// # Arguments
/// * `config` - The configuration to format
/// * `key` - Optional key to filter output (e.g., "mcpServers.npx.enabled")
pub fn format_table(config: &ClaudeConfig, key: Option<&str>) -> Result<()> {
    let json_value = serde_json::to_value(config)?;

    if let Some(key_path) = key {
        // Show specific key
        let value = get_nested_value(&json_value, key_path).unwrap_or(Value::Null);

        println!("{key_path}:");
        print_value(&value, 1);
    } else {
        // Show all configuration
        println!("Claude Code Configuration:");
        println!();

        // Display each section
        if let Some(servers) = &config.mcp_servers {
            println!("MCP Servers:");
            for (name, server) in servers {
                println!("  {name}:");
                println!("    Enabled: {}", server.enabled);
                if let Some(command) = &server.command {
                    println!("    Command: {command}");
                }
                if !server.args.is_empty() {
                    println!("    Args: {}", server.args.join(" "));
                }
            }
            println!();
        }

        if let Some(paths) = &config.allowed_paths {
            println!("Allowed Paths:");
            for path in paths {
                println!("  - {path}");
            }
            println!();
        }

        if let Some(skills) = &config.skills {
            println!("Skills:");
            for (name, skill) in skills {
                println!("  {name}:");
                println!("    Enabled: {}", skill.enabled);
                if let Some(params) = &skill.parameters {
                    println!("    Parameters: {params}");
                }
            }
            println!();
        }

        if let Some(instructions) = &config.custom_instructions {
            println!("Custom Instructions:");
            for (i, instruction) in instructions.iter().enumerate() {
                println!("  {}. {}", i + 1, instruction);
            }
            println!();
        }

        // Show unknown fields
        if !config.unknown.is_empty() {
            println!("Other Configuration:");
            for (key, value) in &config.unknown {
                println!("  {key}:");
                print_value(value, 2);
            }
        }
    }

    Ok(())
}

/// Print a JSON value with indentation
fn print_value(value: &Value, indent: usize) {
    let indent_str = "  ".repeat(indent);

    match value {
        Value::Null => println!("{indent_str}null"),
        Value::Bool(b) => println!("{indent_str}{b}"),
        Value::Number(n) => println!("{indent_str}{n}"),
        Value::String(s) => println!("{indent_str}{s}"),
        Value::Array(arr) => {
            for item in arr {
                print_value(item, indent);
            }
        }
        Value::Object(obj) => {
            for (key, val) in obj {
                println!("{indent_str}{key}:");
                print_value(val, indent + 1);
            }
        }
    }
}

/// Get a nested value from JSON using dot notation
fn get_nested_value(json: &Value, key_path: &str) -> Option<Value> {
    let keys: Vec<&str> = key_path.split('.').collect();
    let mut current = json;

    for key in keys {
        match current {
            Value::Object(map) => {
                current = map.get(key)?;
            }
            Value::Array(arr) => {
                let index = key.parse::<usize>().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format_table_full_config() {
        let config = ClaudeConfig::new().with_custom_instruction("Be concise");

        // Should not panic
        format_table(&config, None).unwrap();
    }

    #[test]
    fn test_format_table_with_key() {
        let config = ClaudeConfig::new().with_custom_instruction("Be concise");

        // Should not panic even with unknown key
        format_table(&config, Some("customInstructions")).unwrap();
    }

    #[test]
    fn test_print_value_string() {
        let value = Value::String("test".to_string());
        // Should not panic
        print_value(&value, 0);
    }

    #[test]
    fn test_print_value_object() {
        let value = json!({
            "key": "value"
        });
        // Should not panic
        print_value(&value, 0);
    }

    #[test]
    fn test_print_value_array() {
        let value = json!(["item1", "item2"]);
        // Should not panic
        print_value(&value, 0);
    }
}
