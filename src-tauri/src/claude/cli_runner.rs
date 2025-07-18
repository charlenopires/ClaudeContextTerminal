use std::process::Command;
use anyhow::Result;

pub struct ClaudeCliRunner;

impl ClaudeCliRunner {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn check_claude_available(&self) -> Result<bool> {
        match Command::new("claude").arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }
    
    pub async fn execute_command(&self, command: &str, args: &[&str]) -> Result<String> {
        let output = Command::new("claude")
            .arg(command)
            .args(args)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Claude command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }
    
    pub async fn start_interactive_session(&self, directory: &str) -> Result<String> {
        // For now, just return a mock session ID
        // In a real implementation, this would start a persistent Claude session
        Ok(format!("claude-session-{}", uuid::Uuid::new_v4()))
    }
}