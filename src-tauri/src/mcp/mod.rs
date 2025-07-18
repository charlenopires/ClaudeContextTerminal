pub mod server_registry;
pub mod hook_manager;
pub mod prompt_enhancer;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServer {
    pub name: String,
    pub description: String,
    pub command: String,
    pub triggers: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHook {
    pub id: String,
    pub name: String,
    pub trigger: HookTrigger,
    pub template: String,
    pub mcp_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTrigger {
    pub trigger_type: TriggerType,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    Keyword,
    FilePattern,
    TaskTag,
}