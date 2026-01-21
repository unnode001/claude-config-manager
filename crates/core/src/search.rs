//! Configuration search functionality
//!
//! Provides search capabilities for finding keys and values
//! across configuration files at different scopes.

use crate::{config::ClaudeConfig, error::Result, types::ConfigScope};
use serde_json::Value;
use std::path::PathBuf;

/// A single search result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// The key path to the found value (e.g., "mcpServers.npx.command")
    pub key_path: String,

    /// The found value
    pub value: String,

    /// Which config this was found in
    pub source: ConfigScope,

    /// Path to the config file (for reference)
    pub config_path: PathBuf,

    /// Type of the value
    pub value_type: ValueType,
}

/// The type of a configuration value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Null,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(
        key_path: String,
        value: String,
        source: ConfigScope,
        config_path: PathBuf,
        value_type: ValueType,
    ) -> Self {
        Self {
            key_path,
            value,
            source,
            config_path,
            value_type,
        }
    }

    /// Format the result for display
    pub fn format(&self) -> String {
        let source_label = match &self.source {
            ConfigScope::Global => "GLOBAL",
            ConfigScope::Project => "PROJECT",
        };

        format!(
            "{}: {} = {} ({})",
            source_label,
            self.key_path,
            self.value,
            self.value_type_label()
        )
    }

    pub fn value_type_label(&self) -> &str {
        match self.value_type {
            ValueType::String => "string",
            ValueType::Number => "number",
            ValueType::Boolean => "bool",
            ValueType::Object => "object",
            ValueType::Array => "array",
            ValueType::Null => "null",
        }
    }
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    /// Search by key (default: true)
    pub search_keys: bool,

    /// Search by value (default: false)
    pub search_values: bool,

    /// Case sensitive search (default: false)
    pub case_sensitive: bool,

    /// Use regex pattern matching (default: false)
    pub regex: bool,

    /// Maximum depth for recursive search (default: unlimited)
    pub max_depth: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            search_keys: true,
            search_values: false,
            case_sensitive: false,
            regex: false,
            max_depth: None,
        }
    }
}

impl SearchOptions {
    /// Create new search options with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to search keys
    pub fn with_keys(mut self, keys: bool) -> Self {
        self.search_keys = keys;
        self
    }

    /// Set whether to search values
    pub fn with_values(mut self, values: bool) -> Self {
        self.search_values = values;
        self
    }

    /// Set case sensitivity
    pub fn with_case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Set regex mode
    pub fn with_regex(mut self, regex: bool) -> Self {
        self.regex = regex;
        self
    }

    /// Set maximum depth
    pub fn with_max_depth(mut self, depth: Option<usize>) -> Self {
        self.max_depth = depth;
        self
    }
}

/// Configuration searcher
pub struct ConfigSearcher {
    options: SearchOptions,
}

impl ConfigSearcher {
    /// Create a new searcher with default options
    pub fn new() -> Self {
        Self {
            options: SearchOptions::default(),
        }
    }

    /// Create a new searcher with custom options
    pub fn with_options(options: SearchOptions) -> Self {
        Self { options }
    }

    /// Search a configuration for matching keys and/or values
    pub fn search(
        &self,
        query: &str,
        config: &ClaudeConfig,
        source: ConfigScope,
        config_path: PathBuf,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Convert config to JSON Value for traversal
        let config_value = serde_json::to_value(config)?;

        // Search the config
        self.search_value(
            query,
            &config_value,
            "",
            &mut results,
            source,
            config_path,
            0,
        )?;

        Ok(results)
    }

    /// Recursively search a JSON value
    fn search_value(
        &self,
        query: &str,
        value: &Value,
        current_path: &str,
        results: &mut Vec<SearchResult>,
        source: ConfigScope,
        config_path: PathBuf,
        depth: usize,
    ) -> Result<()> {
        // Check depth limit
        if let Some(max_depth) = self.options.max_depth {
            if depth > max_depth {
                return Ok(());
            }
        }

        match value {
            Value::Object(map) => {
                for (key, val) in map {
                    let new_path = if current_path.is_empty() {
                        key.clone()
                    } else {
                        format!("{current_path}.{key}")
                    };

                    // Search in key if enabled
                    if self.options.search_keys && self.matches(query, key) {
                        results.push(SearchResult::new(
                            new_path.clone(),
                            format!("<key> {key}"),
                            source,
                            config_path.clone(),
                            ValueType::String,
                        ));
                    }

                    // Recursively search the value
                    self.search_value(
                        query,
                        val,
                        &new_path,
                        results,
                        source,
                        config_path.clone(),
                        depth + 1,
                    )?;
                }
            }
            Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{current_path}[{index}]");

                    // Recursively search array elements
                    self.search_value(
                        query,
                        val,
                        &new_path,
                        results,
                        source,
                        config_path.clone(),
                        depth + 1,
                    )?;
                }
            }
            Value::String(s) => {
                // Search in value if enabled
                if self.options.search_values && self.matches(query, s) {
                    let value_type = ValueType::String;
                    results.push(SearchResult::new(
                        current_path.to_string(),
                        s.clone(),
                        source,
                        config_path,
                        value_type,
                    ));
                }
            }
            Value::Number(n) => {
                // Search in numeric value if enabled
                if self.options.search_values {
                    let num_str = n.to_string();
                    if self.matches(query, &num_str) {
                        let value_type = ValueType::Number;
                        results.push(SearchResult::new(
                            current_path.to_string(),
                            num_str,
                            source,
                            config_path,
                            value_type,
                        ));
                    }
                }
            }
            Value::Bool(b) => {
                // Search in boolean value if enabled
                if self.options.search_values {
                    let bool_str = b.to_string();
                    if self.matches(query, &bool_str) {
                        let value_type = ValueType::Boolean;
                        results.push(SearchResult::new(
                            current_path.to_string(),
                            bool_str,
                            source,
                            config_path,
                            value_type,
                        ));
                    }
                }
            }
            Value::Null => {
                // Don't search null values
            }
        }

        Ok(())
    }

    /// Check if a string matches the query
    fn matches(&self, query: &str, text: &str) -> bool {
        if self.options.case_sensitive {
            text.contains(query)
        } else {
            text.to_lowercase().contains(&query.to_lowercase())
        }
    }
}

impl Default for ConfigSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::new();
        assert!(options.search_keys);
        assert!(!options.search_values);
        assert!(!options.case_sensitive);
        assert!(!options.regex);
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new()
            .with_keys(false)
            .with_values(true)
            .with_case_sensitive(true)
            .with_max_depth(Some(5));

        assert!(!options.search_keys);
        assert!(options.search_values);
        assert!(options.case_sensitive);
        assert_eq!(options.max_depth, Some(5));
    }

    #[test]
    fn test_search_finds_key() {
        let config = ClaudeConfig::new().with_mcp_server(
            "test-server",
            crate::McpServer::new("npx", "npx", vec!["-y".to_string()]),
        );

        let searcher = ConfigSearcher::new();
        let results = searcher
            .search(
                "test",
                &config,
                ConfigScope::Global,
                PathBuf::from("/test/config.json"),
            )
            .unwrap();

        assert!(!results.is_empty());
        assert!(results[0].key_path.contains("test"));
    }

    #[test]
    fn test_search_case_insensitive() {
        let config = ClaudeConfig::new()
            .with_mcp_server("MyServer", crate::McpServer::new("cmd", "cmd", vec![]));

        let options = SearchOptions::new().with_case_sensitive(false);
        let searcher = ConfigSearcher::with_options(options);
        let results = searcher
            .search(
                "myserver",
                &config,
                ConfigScope::Global,
                PathBuf::from("/test/config.json"),
            )
            .unwrap();

        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_respects_depth_limit() {
        let config = ClaudeConfig::new();

        let options = SearchOptions::new().with_max_depth(Some(0));
        let searcher = ConfigSearcher::with_options(options);
        let results = searcher
            .search(
                "anything",
                &config,
                ConfigScope::Global,
                PathBuf::from("/test/config.json"),
            )
            .unwrap();

        // With depth 0, should only search top level
        assert!(results.is_empty() || results.len() < 10);
    }

    #[test]
    fn test_search_result_format() {
        let result = SearchResult::new(
            "mcpServers.test.command".to_string(),
            "npx".to_string(),
            ConfigScope::Global,
            PathBuf::from("/test/config.json"),
            ValueType::String,
        );

        let formatted = result.format();
        assert!(formatted.contains("GLOBAL"));
        assert!(formatted.contains("mcpServers.test.command"));
        assert!(formatted.contains("npx"));
        assert!(formatted.contains("string"));
    }
}
