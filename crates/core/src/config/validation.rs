//! Configuration validation system
//!
//! This module provides traits and implementations for validating
//! Claude Code configuration files according to the specification.

use crate::{
    config::ClaudeConfig,
    error::{ConfigError, Result},
};

/// Trait for configuration validation rules
///
/// Each validation rule checks a specific aspect of the configuration
/// and returns a detailed error if validation fails.
pub trait ValidationRule: Send + Sync {
    /// Validate the configuration
    ///
    /// Returns `Ok(())` if validation passes, or a `ConfigError` with
    /// details if validation fails.
    fn validate(&self, config: &ClaudeConfig) -> Result<()>;

    /// Get the name of this validation rule
    fn name(&self) -> &'static str;
}

/// Validate MCP servers configuration
///
/// Ensures:
/// - Server names are unique
/// - All servers have required fields (enabled)
/// - Server names are not empty
#[derive(Debug, Clone, Default)]
pub struct McpServersRule;

impl ValidationRule for McpServersRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<()> {
        let servers = config.mcp_servers.as_ref();

        // No servers is valid
        let servers = match servers {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(()),
        };

        // Check each server
        for name in servers.keys() {
            // Name should not be empty
            if name.is_empty() {
                return Err(ConfigError::validation_failed(
                    "McpServersRule",
                    "Server name is empty",
                    "All MCP servers must have a non-empty name",
                ));
            }

            // Enabled field must be present (it's required, serde ensures this)
            // Additional validation can be added here
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "McpServersRule"
    }
}

/// Validate allowed paths configuration
///
/// Ensures:
/// - All paths are strings
/// - Paths are syntactically valid (don't contain obvious errors)
#[derive(Debug, Clone, Default)]
pub struct AllowedPathsRule;

impl ValidationRule for AllowedPathsRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<()> {
        let paths = config.allowed_paths.as_ref();

        // No paths is valid
        let paths = match paths {
            Some(p) if !p.is_empty() => p,
            _ => return Ok(()),
        };

        // Check each path
        for (idx, path) in paths.iter().enumerate() {
            // Path should not be empty
            if path.is_empty() {
                return Err(ConfigError::validation_failed(
                    "AllowedPathsRule",
                    format!("Path at index {idx} is empty"),
                    "All allowed paths must be non-empty strings",
                ));
            }

            // Path should not contain null characters
            if path.contains('\0') {
                return Err(ConfigError::validation_failed(
                    "AllowedPathsRule",
                    format!("Path '{path}' contains null character"),
                    "Paths must be valid strings without null characters",
                ));
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "AllowedPathsRule"
    }
}

/// Validate skills configuration
///
/// Ensures:
/// - Skill names are unique (ensured by HashMap)
/// - All skills have required fields (enabled)
/// - Skill names are not empty
#[derive(Debug, Clone, Default)]
pub struct SkillsRule;

impl ValidationRule for SkillsRule {
    fn validate(&self, config: &ClaudeConfig) -> Result<()> {
        let skills = config.skills.as_ref();

        // No skills is valid
        let skills = match skills {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(()),
        };

        // Check each skill
        for name in skills.keys() {
            // Name should not be empty
            if name.is_empty() {
                return Err(ConfigError::validation_failed(
                    "SkillsRule",
                    "Skill name is empty",
                    "All skills must have a non-empty name",
                ));
            }

            // Enabled field must be present (it's required, serde ensures this)
            // Additional validation can be added here
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "SkillsRule"
    }
}

/// Validate all aspects of the configuration
///
/// Runs all validation rules and returns the first error encountered
pub fn validate_config(config: &ClaudeConfig) -> Result<()> {
    let rules: Vec<Box<dyn ValidationRule>> = vec![
        Box::<McpServersRule>::default(),
        Box::<AllowedPathsRule>::default(),
        Box::<SkillsRule>::default(),
    ];

    for rule in rules {
        rule.validate(config)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{McpServer, Skill};

    // TDD Test 1: Empty config is valid
    #[test]
    fn test_empty_config_is_valid() {
        let config = ClaudeConfig::new();
        assert!(validate_config(&config).is_ok());
    }

    // TDD Test 2: Valid MCP server configuration
    #[test]
    fn test_valid_mcp_servers() {
        let server = McpServer::new("npx", "npx", vec![]);
        let config = ClaudeConfig::new().with_mcp_server("npx", server);
        assert!(validate_config(&config).is_ok());
    }

    // TDD Test 3: Invalid MCP server with empty name
    #[test]
    fn test_invalid_mcp_server_empty_name() {
        let mut config = ClaudeConfig::new();
        let mut servers = std::collections::HashMap::new();
        servers.insert("".to_string(), McpServer::new("", "npx", vec![]));
        config.mcp_servers = Some(servers);

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("McpServersRule"));
    }

    // TDD Test 4: Valid allowed paths
    #[test]
    fn test_valid_allowed_paths() {
        let config = ClaudeConfig::new()
            .with_allowed_path("~/projects")
            .with_allowed_path("/usr/local");
        assert!(validate_config(&config).is_ok());
    }

    // TDD Test 5: Invalid allowed path (empty)
    #[test]
    fn test_invalid_allowed_path_empty() {
        let mut config = ClaudeConfig::new();
        config.allowed_paths = Some(vec!["".to_string()]);

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("AllowedPathsRule"));
    }

    // TDD Test 6: Invalid allowed path (null character)
    #[test]
    fn test_invalid_allowed_path_null_character() {
        let mut config = ClaudeConfig::new();
        config.allowed_paths = Some(vec!["/path\0with\0nulls".to_string()]);

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null character"));
    }

    // TDD Test 7: Valid skills configuration
    #[test]
    fn test_valid_skills() {
        let skill = Skill {
            name: "code-review".to_string(),
            enabled: true,
            parameters: Some(serde_json::json!({"strictness": "high"})),
        };
        let config = ClaudeConfig::new().with_skill("code-review", skill);
        assert!(validate_config(&config).is_ok());
    }

    // TDD Test 8: Invalid skill with empty name
    #[test]
    fn test_invalid_skill_empty_name() {
        let mut config = ClaudeConfig::new();
        let mut skills = std::collections::HashMap::new();
        skills.insert(
            "".to_string(),
            Skill {
                name: "".to_string(),
                enabled: true,
                parameters: None,
            },
        );
        config.skills = Some(skills);

        let result = validate_config(&config);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("SkillsRule"));
    }

    // TDD Test 9: All rules pass on valid config
    #[test]
    fn test_all_rules_pass() {
        let server = McpServer::new("npx", "npx", vec![]);
        let skill = Skill {
            name: "test".to_string(),
            enabled: true,
            parameters: None,
        };
        let config = ClaudeConfig::new()
            .with_mcp_server("npx", server)
            .with_allowed_path("~/projects")
            .with_skill("test", skill);

        assert!(validate_config(&config).is_ok());
    }

    // TDD Test 10: Validation provides helpful error messages
    #[test]
    fn test_validation_error_messages_are_helpful() {
        let mut config = ClaudeConfig::new();
        config.allowed_paths = Some(vec!["".to_string()]);

        let result = validate_config(&config);
        assert!(result.is_err());

        let err = result.unwrap_err().to_string();
        assert!(err.contains("AllowedPathsRule"));
        assert!(err.contains("Suggestion:"));
    }
}
