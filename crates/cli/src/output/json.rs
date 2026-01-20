//! JSON output formatter
//!
//! Formats configuration as JSON

use anyhow::Result;
use claude_config_manager_core::ClaudeConfig;
use serde_json::Value;

/// Format configuration as JSON
///
/// # Arguments
/// * `config` - The configuration to format
/// * `key` - Optional key to filter output (e.g., "mcpServers.npx.enabled")
pub fn format_json(config: &ClaudeConfig, key: Option<&str>) -> Result<()> {
    // Convert config to JSON value
    let json_value = serde_json::to_value(config)?;

    // Filter by key if specified
    let output = if let Some(key_path) = key {
        get_nested_value(&json_value, key_path)
            .unwrap_or_else(|| Value::Null)
    } else {
        json_value
    };

    // Pretty print JSON
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Get a nested value from JSON using dot notation
///
/// # Arguments
/// * `json` - The JSON value to search
/// * `key_path` - Dot-separated key path (e.g., "mcpServers.npx.enabled")
fn get_nested_value(json: &Value, key_path: &str) -> Option<Value> {
    let keys: Vec<&str> = key_path.split('.').collect();
    let mut current = json;

    for key in keys {
        match current {
            Value::Object(map) => {
                current = map.get(key)?;
            }
            Value::Array(arr) => {
                // Try to parse as index
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
    fn test_format_json_full_config() {
        let config = ClaudeConfig::new()
            .with_custom_instruction("Be concise");

        // Should not panic
        format_json(&config, None).unwrap();
    }

    #[test]
    fn test_get_nested_value_simple() {
        let json = json!({
            "mcpServers": {
                "npx": {
                    "enabled": true
                }
            }
        });

        let result = get_nested_value(&json, "mcpServers");
        assert!(result.is_some());
        assert!(result.unwrap().is_object());
    }

    #[test]
    fn test_get_nested_value_deep() {
        let json = json!({
            "mcpServers": {
                "npx": {
                    "enabled": true
                }
            }
        });

        let result = get_nested_value(&json, "mcpServers.npx.enabled");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), json!(true));
    }

    #[test]
    fn test_get_nested_value_missing_key() {
        let json = json!({
            "mcpServers": {
                "npx": {
                    "enabled": true
                }
            }
        });

        let result = get_nested_value(&json, "mcpServers.nonexistent.enabled");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_nested_value_from_array() {
        let json = json!({
            "allowedPaths": ["~/projects", "~/work"]
        });

        let result = get_nested_value(&json, "allowedPaths.0");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), json!("~/projects"));
    }
}
