pub mod cli_runner;
pub mod session_manager;
pub mod prompt_builder;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSession {
    pub id: String,
    pub status: SessionStatus,
    pub current_directory: String,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub total_commands: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Stopped,
    Error(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskExecutionRequest {
    pub task_id: String,
    pub task_title: String,
    pub task_description: String,
    pub task_prompt: String,
    pub context_files: Vec<String>,
    pub mcp_servers: Vec<String>,
}