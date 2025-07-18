use crate::mcp::{MCPServer, PromptHook, HookTrigger, TriggerType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

static mut MCP_SERVERS: Option<Arc<Mutex<HashMap<String, MCPServer>>>> = None;
static mut PROMPT_HOOKS: Option<Arc<Mutex<HashMap<String, PromptHook>>>> = None;

fn get_mcp_servers() -> &'static Arc<Mutex<HashMap<String, MCPServer>>> {
    unsafe {
        if MCP_SERVERS.is_none() {
            let mut servers = HashMap::new();
            
            // Add some default MCP servers
            servers.insert("code-analyzer".to_string(), MCPServer {
                name: "code-analyzer".to_string(),
                description: "Analyze code structure and patterns".to_string(),
                command: "mcp-code-analyzer".to_string(),
                triggers: vec!["analyze".to_string(), "review".to_string()],
                enabled: true,
            });
            
            servers.insert("test-generator".to_string(), MCPServer {
                name: "test-generator".to_string(),
                description: "Generate comprehensive tests".to_string(),
                command: "mcp-test-generator".to_string(),
                triggers: vec!["test".to_string(), "spec".to_string()],
                enabled: true,
            });
            
            MCP_SERVERS = Some(Arc::new(Mutex::new(servers)));
        }
        MCP_SERVERS.as_ref().unwrap()
    }
}

fn get_prompt_hooks() -> &'static Arc<Mutex<HashMap<String, PromptHook>>> {
    unsafe {
        if PROMPT_HOOKS.is_none() {
            PROMPT_HOOKS = Some(Arc::new(Mutex::new(HashMap::new())));
        }
        PROMPT_HOOKS.as_ref().unwrap()
    }
}

#[tauri::command]
pub async fn list_servers() -> Result<Vec<MCPServer>, String> {
    let servers = get_mcp_servers();
    if let Ok(servers_map) = servers.lock() {
        Ok(servers_map.values().cloned().collect())
    } else {
        Err("Failed to access MCP servers".to_string())
    }
}

#[tauri::command]
pub async fn toggle_server(server_name: String, enabled: bool) -> Result<(), String> {
    let servers = get_mcp_servers();
    if let Ok(mut servers_map) = servers.lock() {
        if let Some(server) = servers_map.get_mut(&server_name) {
            server.enabled = enabled;
            Ok(())
        } else {
            Err("Server not found".to_string())
        }
    } else {
        Err("Failed to access MCP servers".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateHookRequest {
    pub name: String,
    pub trigger_type: TriggerType,
    pub trigger_value: String,
    pub template: String,
    pub mcp_servers: Vec<String>,
}

#[tauri::command]
pub async fn create_hook(request: CreateHookRequest) -> Result<PromptHook, String> {
    let hook = PromptHook {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        trigger: HookTrigger {
            trigger_type: request.trigger_type,
            value: request.trigger_value,
        },
        template: request.template,
        mcp_servers: request.mcp_servers,
    };

    let hooks = get_prompt_hooks();
    if let Ok(mut hooks_map) = hooks.lock() {
        hooks_map.insert(hook.id.clone(), hook.clone());
        Ok(hook)
    } else {
        Err("Failed to create hook".to_string())
    }
}