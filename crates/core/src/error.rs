//! Error types for Claude Config Manager
//!
//! This module defines all error types used throughout the core library.
//! All errors provide clear, actionable messages to help users resolve issues.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Core error type for configuration management operations
///
/// # Design Principles
/// - Every error variant provides clear, actionable guidance
/// - Error messages explain what went wrong AND how to fix it
/// - Never lose user data - prefer refusing operations over corruption
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file not found
    ///
    /// Provides guidance on creating a new config file
    #[error("Configuration file not found: {path}\n\nSuggestion: Create a new config file or specify a valid path with --project")]
    NotFound { path: PathBuf },

    /// Invalid JSON in configuration file
    ///
    /// Includes line number and specific error details
    #[error("Invalid JSON in configuration file: {path}\nError at line {line}, column {column}: {message}\n\nSuggestion: Check JSON syntax and ensure proper quoting")]
    InvalidJson {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Configuration validation failed
    ///
    /// Explains what validation rule was violated
    #[error(
        "Configuration validation failed: {rule}\n\nDetails: {details}\n\nSuggestion: {suggestion}"
    )]
    ValidationFailed {
        rule: String,
        details: String,
        suggestion: String,
    },

    /// Filesystem operation failed
    ///
    /// Provides OS-level error details
    #[error("Filesystem error: {operation} failed for {path}\n\nOS Error: {source}\n\nSuggestion: Check file permissions and disk space")]
    Filesystem {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },

    /// Backup creation failed
    ///
    /// Critical error - operation was aborted to protect user data
    #[error("Backup creation failed for {path}\n\nError: {source}\n\nSuggestion: Ensure sufficient disk space and write permissions. Operation aborted to protect your data.")]
    BackupFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Permission denied
    ///
    /// Provides guidance on fixing permissions
    #[error("Permission denied: {operation} on {path}\n\nSuggestion: Check file permissions. Try running with appropriate privileges or changing file ownership.")]
    PermissionDenied { operation: String, path: PathBuf },

    /// MCP server operation failed
    #[error("MCP server error: {server} - {operation}\n\nDetails: {details}\n\nSuggestion: Check server configuration and ensure the server is accessible.")]
    McpServerError {
        server: String,
        operation: String,
        details: String,
    },

    /// Generic error with context
    #[error("{0}")]
    Generic(String),
}

impl ConfigError {
    /// Create a NotFound error
    pub fn not_found(path: impl Into<PathBuf>) -> Self {
        Self::NotFound { path: path.into() }
    }

    /// Create an InvalidJson error
    pub fn invalid_json(
        path: impl Into<PathBuf>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidJson {
            path: path.into(),
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a ValidationFailed error
    pub fn validation_failed(
        rule: impl Into<String>,
        details: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::ValidationFailed {
            rule: rule.into(),
            details: details.into(),
            suggestion: suggestion.into(),
        }
    }

    /// Create a Filesystem error
    pub fn filesystem(
        operation: impl Into<String>,
        path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::Filesystem {
            operation: operation.into(),
            path: path.into(),
            source,
        }
    }

    /// Create a BackupFailed error
    pub fn backup_failed(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::BackupFailed {
            path: path.into(),
            source,
        }
    }

    /// Create a PermissionDenied error
    pub fn permission_denied(operation: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            path: path.into(),
        }
    }

    /// Create an McpServerError
    pub fn mcp_server_error(
        server: impl Into<String>,
        operation: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self::McpServerError {
            server: server.into(),
            operation: operation.into(),
            details: details.into(),
        }
    }
}

// Implement From conversions for common error types
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Generic(format!("JSON error: {}", err))
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Filesystem {
            operation: "filesystem".to_string(),
            path: PathBuf::from("unknown"),
            source: err,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages_are_actionable() {
        let error = ConfigError::not_found("/test/config.json");
        let message = error.to_string();
        assert!(message.contains("Suggestion:"));
    }

    #[test]
    fn test_invalid_json_error() {
        let error = ConfigError::invalid_json("/test/config.json", 10, 5, "Unexpected token");
        let message = format!("{}", error);
        assert!(message.contains("line 10"));
        assert!(message.contains("Unexpected token"));
    }

    #[test]
    fn test_validation_error() {
        let error = ConfigError::validation_failed(
            "McpServersRule",
            "Server 'test' already exists",
            "Use a different server name or remove the existing server first",
        );
        let message = format!("{}", error);
        assert!(message.contains("McpServersRule"));
        assert!(message.contains("Use a different server name"));
    }

    #[test]
    fn test_backup_failed_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let error = ConfigError::backup_failed("/test/config.json", io_error);
        let message = format!("{}", error);
        assert!(message.contains("Operation aborted"));
        assert!(message.contains("protect your data"));
    }
}
