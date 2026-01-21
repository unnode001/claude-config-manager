//! Data types for Tauri commands

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration data for display in GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfigData {
    pub mcp_servers: Option<HashMap<String, McpServerData>>,
    pub skills: Option<HashMap<String, SkillData>>,
    pub allowed_paths: Option<Vec<String>>,
    pub custom_instructions: Option<Vec<String>>,
    pub unknown: HashMap<String, serde_json::Value>,
}

impl From<claude_config_manager_core::ClaudeConfig> for ClaudeConfigData {
    fn from(config: claude_config_manager_core::ClaudeConfig) -> Self {
        Self {
            mcp_servers: config.mcp_servers.map(|servers| {
                servers
                    .into_iter()
                    .map(|(k, v)| (k, McpServerData::from(v)))
                    .collect()
            }),
            skills: config.skills.map(|skills| {
                skills
                    .into_iter()
                    .map(|(k, v)| (k, SkillData::from(v)))
                    .collect()
            }),
            allowed_paths: config.allowed_paths,
            custom_instructions: config.custom_instructions,
            unknown: config.unknown,
        }
    }
}

/// MCP server data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerData {
    pub name: String,
    pub enabled: bool,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

impl From<claude_config_manager_core::McpServer> for McpServerData {
    fn from(server: claude_config_manager_core::McpServer) -> Self {
        Self {
            name: server.name,
            enabled: server.enabled,
            command: server.command.unwrap_or_default(),
            args: server.args,
            env: server.env,
        }
    }
}

/// Skill data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillData {
    pub name: String,
    pub enabled: bool,
    pub parameters: Option<serde_json::Value>,
}

impl From<claude_config_manager_core::Skill> for SkillData {
    fn from(skill: claude_config_manager_core::Skill) -> Self {
        Self {
            name: skill.name,
            enabled: skill.enabled,
            parameters: skill.parameters,
        }
    }
}

/// Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: String,
    pub path: String,
    pub root: String,
    pub claude_dir: String,
    pub has_config: bool,
}

impl From<claude_config_manager_core::ProjectInfo> for ProjectData {
    fn from(info: claude_config_manager_core::ProjectInfo) -> Self {
        Self {
            name: info.name,
            path: info.root.to_string_lossy().to_string(),
            root: info.root.to_string_lossy().to_string(),
            claude_dir: info.claude_dir.to_string_lossy().to_string(),
            has_config: info.has_config,
        }
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultData {
    pub path: String,
    pub value: String,
    pub source: String,
    pub config_path: String,
    pub value_type: String,
}

impl From<claude_config_manager_core::SearchResult> for SearchResultData {
    fn from(result: claude_config_manager_core::SearchResult) -> Self {
        Self {
            path: result.key_path,
            value: result.value,
            source: format!("{:?}", result.source),
            config_path: result.config_path.to_string_lossy().to_string(),
            value_type: format!("{:?}", result.value_type),
        }
    }
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfoData {
    pub path: String,
    pub original_path: String,
    pub created_at: String,
    pub size: u64,
}

impl From<claude_config_manager_core::BackupInfo> for BackupInfoData {
    fn from(info: claude_config_manager_core::BackupInfo) -> Self {
        Self {
            path: info.path,
            original_path: info.original_path,
            created_at: info.created_at.to_rfc3339(),
            size: info.size,
        }
    }
}
