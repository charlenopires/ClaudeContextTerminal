//! Safety validation utilities for tools

use super::{ToolRequest, ToolResult};
use std::path::Path;

/// Safety validator for tool operations
pub struct SafeValidator;

impl SafeValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate if a file path is safe for operations
    pub fn validate_path(&self, path: &str, permissions: &super::ToolPermissions) -> ToolResult<()> {
        let path_obj = Path::new(path);

        // Must be absolute path
        if !path_obj.is_absolute() {
            return Err(anyhow::anyhow!("Path must be absolute"));
        }

        // Check restricted paths
        for restricted in &permissions.restricted_paths {
            if path.starts_with(restricted) && !permissions.yolo_mode {
                return Err(anyhow::anyhow!("Access to path '{}' is restricted", path));
            }
        }

        // Check for path traversal attempts
        if path.contains("..") && !permissions.yolo_mode {
            return Err(anyhow::anyhow!("Path traversal detected in: {}", path));
        }

        Ok(())
    }

    /// Validate if a command is safe to execute
    pub fn validate_command(&self, command: &str, permissions: &super::ToolPermissions) -> ToolResult<()> {
        if !permissions.allow_execute && !permissions.yolo_mode {
            return Err(anyhow::anyhow!("Command execution not permitted"));
        }

        // Check for dangerous commands
        let dangerous_patterns = [
            "rm -rf", "mkfs", "dd if=", "shutdown", "reboot",
            "chmod 777", "chown root", ":(){ :|:& };:",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) && !permissions.yolo_mode {
                return Err(anyhow::anyhow!("Potentially dangerous command detected: {}", pattern));
            }
        }

        Ok(())
    }

    /// Validate tool request parameters
    pub fn validate_request(&self, request: &ToolRequest) -> ToolResult<()> {
        // Basic request validation
        if request.tool_name.is_empty() {
            return Err(anyhow::anyhow!("Tool name cannot be empty"));
        }

        // Validate file paths if present
        if let Some(file_path) = request.parameters.get("file_path").and_then(|v| v.as_str()) {
            self.validate_path(file_path, &request.permissions)?;
        }

        if let Some(path) = request.parameters.get("path").and_then(|v| v.as_str()) {
            self.validate_path(path, &request.permissions)?;
        }

        // Validate commands if present
        if let Some(command) = request.parameters.get("command").and_then(|v| v.as_str()) {
            self.validate_command(command, &request.permissions)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tools::ToolPermissions;

    #[test]
    fn test_path_validation() {
        let validator = SafeValidator::new();
        let permissions = ToolPermissions::default();

        // Valid absolute path
        assert!(validator.validate_path("/tmp/test.txt", &permissions).is_ok());

        // Invalid relative path
        assert!(validator.validate_path("test.txt", &permissions).is_err());

        // Path traversal
        assert!(validator.validate_path("/tmp/../etc/passwd", &permissions).is_err());

        // Restricted path
        assert!(validator.validate_path("/etc/passwd", &permissions).is_err());
    }

    #[test]
    fn test_command_validation() {
        let validator = SafeValidator::new();
        let mut permissions = ToolPermissions::default();

        // No execute permission
        assert!(validator.validate_command("ls -la", &permissions).is_err());

        // With execute permission
        permissions.allow_execute = true;
        assert!(validator.validate_command("ls -la", &permissions).is_ok());

        // Dangerous command
        assert!(validator.validate_command("rm -rf /", &permissions).is_err());

        // YOLO mode overrides
        permissions.yolo_mode = true;
        assert!(validator.validate_command("rm -rf /", &permissions).is_ok());
    }
}