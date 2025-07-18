use super::{MCPServer, PromptHook};
use crate::mcp::hook_manager::{HookManager, HookContext};

pub struct PromptEnhancer {
    hook_manager: HookManager,
}

impl PromptEnhancer {
    pub fn new() -> Self {
        Self {
            hook_manager: HookManager::new(),
        }
    }
    
    pub fn enhance_prompt(
        &self,
        base_prompt: &str,
        context: &HookContext,
        available_servers: &[MCPServer],
    ) -> String {
        let mut enhanced_prompt = base_prompt.to_string();
        
        let matching_hooks = self.hook_manager.find_matching_hooks(context);
        
        if !matching_hooks.is_empty() {
            enhanced_prompt.push_str("\n\n[CONTEXTO AUTOMÁTICO]\n");
            
            for hook in matching_hooks {
                enhanced_prompt.push_str(&format!("- {}\n", hook.template));
                
                // Add relevant MCP servers
                for server_name in &hook.mcp_servers {
                    if let Some(server) = available_servers.iter().find(|s| &s.name == server_name) {
                        if server.enabled {
                            enhanced_prompt.push_str(&format!("  - Use @{}: {}\n", server.name, server.description));
                        }
                    }
                }
            }
        }
        
        enhanced_prompt
    }
    
    pub fn add_hook(&mut self, hook: PromptHook) {
        self.hook_manager.add_hook(hook);
    }
    
    pub fn remove_hook(&mut self, id: &str) -> Option<PromptHook> {
        self.hook_manager.remove_hook(id)
    }
    
    pub fn list_hooks(&self) -> Vec<&PromptHook> {
        self.hook_manager.list_hooks()
    }
    
    pub fn suggest_servers(&self, prompt: &str, available_servers: &[MCPServer]) -> Vec<&MCPServer> {
        let prompt_lower = prompt.to_lowercase();
        
        available_servers.iter()
            .filter(|server| {
                server.enabled && server.triggers.iter().any(|trigger| prompt_lower.contains(trigger))
            })
            .collect()
    }
}