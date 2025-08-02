//! Permission management system for controlling tool access

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub mod validator;
pub mod manager;

pub use validator::PermissionValidator;
pub use manager::PermissionManager;

/// Permission levels for different types of operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Read-only operations (file reading, directory listing)
    Read,
    /// File modifications (editing, creating files)
    Write,
    /// Command execution
    Execute,
    /// Network access
    Network,
    /// Potentially dangerous operations
    Dangerous,
}

/// Permission mode for handling requests
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionMode {
    /// Ask for permission before each operation
    Prompt,
    /// Automatically allow the operation
    Auto,
    /// Automatically deny the operation
    Deny,
}

/// Tool-specific permission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPermission {
    pub tool_name: String,
    pub mode: PermissionMode,
    pub allowed_paths: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
    pub max_file_size: Option<u64>, // in bytes
    pub timeout_ms: Option<u64>,
}

impl Default for ToolPermission {
    fn default() -> Self {
        Self {
            tool_name: String::new(),
            mode: PermissionMode::Prompt,
            allowed_paths: Vec::new(),
            denied_paths: Vec::new(),
            max_file_size: Some(10_000_000), // 10MB default
            timeout_ms: Some(30000), // 30 seconds default
        }
    }
}

/// Global permission configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// Whether YOLO mode is enabled (bypasses all safety checks)
    pub yolo_mode: bool,
    
    /// Default permission mode for unspecified tools
    pub default_mode: PermissionMode,
    
    /// Tool-specific permissions
    pub tool_permissions: HashMap<String, ToolPermission>,
    
    /// Globally restricted paths (even in YOLO mode, these require explicit override)
    pub restricted_paths: Vec<PathBuf>,
    
    /// Globally allowed paths (always safe)
    pub safe_paths: Vec<PathBuf>,
    
    /// Maximum file size for operations (bytes)
    pub max_file_size: u64,
    
    /// Default command timeout (milliseconds)
    pub default_timeout_ms: u64,
    
    /// Whether to log all permission decisions
    pub log_decisions: bool,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        let mut tool_permissions = HashMap::new();
        
        // Set up default tool permissions
        tool_permissions.insert("file".to_string(), ToolPermission {
            tool_name: "file".to_string(),
            mode: PermissionMode::Auto,
            ..Default::default()
        });
        
        tool_permissions.insert("ls".to_string(), ToolPermission {
            tool_name: "ls".to_string(),
            mode: PermissionMode::Auto,
            ..Default::default()
        });
        
        tool_permissions.insert("grep".to_string(), ToolPermission {
            tool_name: "grep".to_string(),
            mode: PermissionMode::Auto,
            ..Default::default()
        });
        
        tool_permissions.insert("edit".to_string(), ToolPermission {
            tool_name: "edit".to_string(),
            mode: PermissionMode::Prompt,
            ..Default::default()
        });
        
        tool_permissions.insert("bash".to_string(), ToolPermission {
            tool_name: "bash".to_string(),
            mode: PermissionMode::Prompt,
            timeout_ms: Some(120000), // 2 minutes for commands
            ..Default::default()
        });

        Self {
            yolo_mode: false,
            default_mode: PermissionMode::Prompt,
            tool_permissions,
            restricted_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/sys"),
                PathBuf::from("/proc"),
                PathBuf::from("/dev"),
                PathBuf::from("/root"),
                PathBuf::from("/boot"),
            ],
            safe_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
            max_file_size: 50_000_000, // 50MB
            default_timeout_ms: 30000, // 30 seconds
            log_decisions: true,
        }
    }
}

/// Result of a permission check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    /// Operation is allowed
    Allowed,
    /// Operation is denied
    Denied(String),
    /// User should be prompted for permission
    Prompt(String),
}

/// Context for permission decisions
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub tool_name: String,
    pub operation: String,
    pub file_path: Option<PathBuf>,
    pub command: Option<String>,
    pub file_size: Option<u64>,
    pub risk_level: PermissionLevel,
}

impl PermissionContext {
    pub fn new(tool_name: String, operation: String) -> Self {
        Self {
            tool_name,
            operation,
            file_path: None,
            command: None,
            file_size: None,
            risk_level: PermissionLevel::Read,
        }
    }

    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    pub fn with_command(mut self, command: String) -> Self {
        self.command = Some(command);
        self.risk_level = PermissionLevel::Execute;
        self
    }

    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    pub fn with_risk_level(mut self, level: PermissionLevel) -> Self {
        self.risk_level = level;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_config_default() {
        let config = PermissionConfig::default();
        
        assert!(!config.yolo_mode);
        assert_eq!(config.default_mode, PermissionMode::Prompt);
        assert!(!config.tool_permissions.is_empty());
        
        // Check that safe tools are auto-approved
        assert_eq!(
            config.tool_permissions.get("file").unwrap().mode,
            PermissionMode::Auto
        );
        
        // Check that dangerous tools require prompts
        assert_eq!(
            config.tool_permissions.get("bash").unwrap().mode,
            PermissionMode::Prompt
        );
    }

    #[test]
    fn test_permission_context_builder() {
        let context = PermissionContext::new("test".to_string(), "read".to_string())
            .with_file_path(PathBuf::from("/tmp/test.txt"))
            .with_file_size(1000)
            .with_risk_level(PermissionLevel::Write);
        
        assert_eq!(context.tool_name, "test");
        assert_eq!(context.operation, "read");
        assert_eq!(context.file_path, Some(PathBuf::from("/tmp/test.txt")));
        assert_eq!(context.file_size, Some(1000));
        assert_eq!(context.risk_level, PermissionLevel::Write);
    }

    #[test]
    fn test_tool_permission_default() {
        let perm = ToolPermission::default();
        
        assert_eq!(perm.mode, PermissionMode::Prompt);
        assert_eq!(perm.max_file_size, Some(10_000_000));
        assert_eq!(perm.timeout_ms, Some(30000));
    }
}