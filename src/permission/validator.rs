//! Permission validation logic

use super::{PermissionConfig, PermissionContext, PermissionResult, PermissionLevel, PermissionMode};
use std::path::Path;
use tracing::{debug, warn};

/// Validates permissions for tool operations
pub struct PermissionValidator {
    config: PermissionConfig,
}

impl PermissionValidator {
    /// Create a new permission validator
    pub fn new(config: PermissionConfig) -> Self {
        Self { config }
    }

    /// Check if an operation is permitted
    pub fn check_permission(&self, context: &PermissionContext) -> PermissionResult {
        if self.config.log_decisions {
            debug!("Checking permission for tool '{}' operation '{}'", 
                   context.tool_name, context.operation);
        }

        // YOLO mode bypasses most checks (but not critical system paths)
        if self.config.yolo_mode {
            if let Some(result) = self.check_critical_restrictions(context) {
                return result;
            }
            return PermissionResult::Allowed;
        }

        // Check file path restrictions
        if let Some(file_path) = &context.file_path {
            if let Some(result) = self.check_path_permissions(file_path, context) {
                return result;
            }
        }

        // Check command restrictions
        if let Some(command) = &context.command {
            if let Some(result) = self.check_command_permissions(command, context) {
                return result;
            }
        }

        // Check file size restrictions
        if let Some(size) = context.file_size {
            if let Some(result) = self.check_file_size(size, context) {
                return result;
            }
        }

        // Check tool-specific permissions
        self.check_tool_permissions(context)
    }

    /// Check critical system restrictions that even YOLO mode respects
    fn check_critical_restrictions(&self, context: &PermissionContext) -> Option<PermissionResult> {
        // Check for extremely dangerous operations
        if let Some(command) = &context.command {
            let critical_patterns = [
                "rm -rf /", "rm -rf /*", "mkfs", "fdisk /dev/sd",
                "dd if=/dev/zero of=/dev/sd", ":(){ :|:& };:",
            ];
            
            for pattern in &critical_patterns {
                if command.contains(pattern) {
                    warn!("Critical dangerous operation blocked even in YOLO mode: {}", command);
                    return Some(PermissionResult::Denied(
                        format!("Critical system operation '{}' is blocked even in YOLO mode", pattern)
                    ));
                }
            }
        }

        // Check for writes to critical system files
        if let Some(file_path) = &context.file_path {
            let critical_files = [
                "/etc/passwd", "/etc/shadow", "/etc/sudoers",
                "/boot/grub/grub.cfg", "/etc/fstab",
            ];
            
            for critical_file in &critical_files {
                if file_path == Path::new(critical_file) && context.risk_level == PermissionLevel::Write {
                    warn!("Critical file write blocked even in YOLO mode: {}", file_path.display());
                    return Some(PermissionResult::Denied(
                        format!("Writing to critical system file '{}' is blocked", file_path.display())
                    ));
                }
            }
        }

        None
    }

    /// Check path-based permissions
    fn check_path_permissions(&self, file_path: &Path, context: &PermissionContext) -> Option<PermissionResult> {
        // Check if path is in safe paths (always allowed)
        for safe_path in &self.config.safe_paths {
            if file_path.starts_with(safe_path) {
                return Some(PermissionResult::Allowed);
            }
        }

        // Check if path is restricted
        for restricted_path in &self.config.restricted_paths {
            if file_path.starts_with(restricted_path) {
                return Some(PermissionResult::Denied(
                    format!("Access to restricted path '{}' is not allowed", restricted_path.display())
                ));
            }
        }

        // Check tool-specific path restrictions
        if let Some(tool_perm) = self.config.tool_permissions.get(&context.tool_name) {
            // Check denied paths
            for denied_path in &tool_perm.denied_paths {
                if file_path.starts_with(denied_path) {
                    return Some(PermissionResult::Denied(
                        format!("Tool '{}' is not allowed to access path '{}'", 
                               context.tool_name, denied_path.display())
                    ));
                }
            }

            // Check allowed paths (if any are specified, path must be in the list)
            if !tool_perm.allowed_paths.is_empty() {
                let allowed = tool_perm.allowed_paths.iter()
                    .any(|allowed_path| file_path.starts_with(allowed_path));
                
                if !allowed {
                    return Some(PermissionResult::Denied(
                        format!("Tool '{}' can only access specific allowed paths", context.tool_name)
                    ));
                }
            }
        }

        None
    }

    /// Check command-based permissions
    fn check_command_permissions(&self, command: &str, context: &PermissionContext) -> Option<PermissionResult> {
        // Check for dangerous command patterns
        let dangerous_patterns = [
            ("rm -rf", "Recursive file deletion"),
            ("chmod 777", "Dangerous permission change"),
            ("chown root", "Root ownership change"),
            ("shutdown", "System shutdown"),
            ("reboot", "System reboot"),
            ("mkfs", "Filesystem creation"),
            ("fdisk", "Disk partitioning"),
            ("curl", "Network download"),
            ("wget", "Network download"),
            ("nc ", "Network connection"),
            ("netcat", "Network connection"),
        ];

        for (pattern, description) in &dangerous_patterns {
            if command.contains(pattern) {
                return Some(PermissionResult::Prompt(
                    format!("Command contains potentially dangerous operation '{}' ({}). Allow execution?", 
                           pattern, description)
                ));
            }
        }

        None
    }

    /// Check file size restrictions
    fn check_file_size(&self, size: u64, context: &PermissionContext) -> Option<PermissionResult> {
        let max_size = if let Some(tool_perm) = self.config.tool_permissions.get(&context.tool_name) {
            tool_perm.max_file_size.unwrap_or(self.config.max_file_size)
        } else {
            self.config.max_file_size
        };

        if size > max_size {
            return Some(PermissionResult::Denied(
                format!("File size {} bytes exceeds maximum allowed size {} bytes", size, max_size)
            ));
        }

        None
    }

    /// Check tool-specific permissions
    fn check_tool_permissions(&self, context: &PermissionContext) -> PermissionResult {
        let tool_perm = self.config.tool_permissions.get(&context.tool_name);
        let mode = tool_perm.map(|p| &p.mode).unwrap_or(&self.config.default_mode);

        match mode {
            PermissionMode::Auto => PermissionResult::Allowed,
            PermissionMode::Deny => PermissionResult::Denied(
                format!("Tool '{}' is explicitly denied", context.tool_name)
            ),
            PermissionMode::Prompt => {
                let risk_description = match context.risk_level {
                    PermissionLevel::Read => "read data",
                    PermissionLevel::Write => "modify files",
                    PermissionLevel::Execute => "execute commands",
                    PermissionLevel::Network => "access network",
                    PermissionLevel::Dangerous => "perform dangerous operations",
                };

                PermissionResult::Prompt(
                    format!("Tool '{}' wants to {}. Allow operation?", 
                           context.tool_name, risk_description)
                )
            }
        }
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: PermissionConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PermissionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_yolo_mode_allows_most_operations() {
        let mut config = PermissionConfig::default();
        config.yolo_mode = true;
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("test".to_string(), "read".to_string())
            .with_file_path(PathBuf::from("/tmp/test.txt"));

        assert_eq!(validator.check_permission(&context), PermissionResult::Allowed);
    }

    #[test]
    fn test_critical_operations_blocked_even_in_yolo() {
        let mut config = PermissionConfig::default();
        config.yolo_mode = true;
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("bash".to_string(), "execute".to_string())
            .with_command("rm -rf /".to_string());

        match validator.check_permission(&context) {
            PermissionResult::Denied(_) => (), // Expected
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    #[test]
    fn test_safe_paths_always_allowed() {
        let config = PermissionConfig::default();
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("edit".to_string(), "write".to_string())
            .with_file_path(PathBuf::from("/tmp/test.txt"))
            .with_risk_level(PermissionLevel::Write);

        assert_eq!(validator.check_permission(&context), PermissionResult::Allowed);
    }

    #[test]
    fn test_restricted_paths_denied() {
        let config = PermissionConfig::default();
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("edit".to_string(), "write".to_string())
            .with_file_path(PathBuf::from("/etc/passwd"))
            .with_risk_level(PermissionLevel::Write);

        match validator.check_permission(&context) {
            PermissionResult::Denied(_) => (), // Expected
            other => panic!("Expected Denied, got {:?}", other),
        }
    }

    #[test]
    fn test_auto_mode_tools() {
        let config = PermissionConfig::default();
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("file".to_string(), "read".to_string())
            .with_file_path(PathBuf::from("/home/user/test.txt"));

        assert_eq!(validator.check_permission(&context), PermissionResult::Allowed);
    }

    #[test]
    fn test_prompt_mode_tools() {
        let config = PermissionConfig::default();
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("bash".to_string(), "execute".to_string())
            .with_command("ls -la".to_string())
            .with_risk_level(PermissionLevel::Execute);

        match validator.check_permission(&context) {
            PermissionResult::Prompt(_) => (), // Expected
            other => panic!("Expected Prompt, got {:?}", other),
        }
    }

    #[test]
    fn test_file_size_limits() {
        let config = PermissionConfig::default();
        let validator = PermissionValidator::new(config);

        let context = PermissionContext::new("file".to_string(), "read".to_string())
            .with_file_size(100_000_000); // 100MB, exceeds default 50MB limit

        match validator.check_permission(&context) {
            PermissionResult::Denied(_) => (), // Expected
            other => panic!("Expected Denied, got {:?}", other),
        }
    }
}