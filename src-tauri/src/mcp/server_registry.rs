use super::MCPServer;
use std::collections::HashMap;

pub struct ServerRegistry {
    servers: HashMap<String, MCPServer>,
}

impl ServerRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            servers: HashMap::new(),
        };
        
        registry.register_default_servers();
        registry
    }
    
    fn register_default_servers(&mut self) {
        self.servers.insert("code-analyzer".to_string(), MCPServer {
            name: "code-analyzer".to_string(),
            description: "Analyze code structure and patterns".to_string(),
            command: "mcp-code-analyzer".to_string(),
            triggers: vec!["analyze".to_string(), "review".to_string()],
            enabled: true,
        });
        
        self.servers.insert("test-generator".to_string(), MCPServer {
            name: "test-generator".to_string(),
            description: "Generate comprehensive tests".to_string(),
            command: "mcp-test-generator".to_string(),
            triggers: vec!["test".to_string(), "spec".to_string()],
            enabled: true,
        });
        
        self.servers.insert("doc-writer".to_string(), MCPServer {
            name: "doc-writer".to_string(),
            description: "Generate documentation".to_string(),
            command: "mcp-doc-writer".to_string(),
            triggers: vec!["docs".to_string(), "documentation".to_string()],
            enabled: false,
        });
    }
    
    pub fn get_server(&self, name: &str) -> Option<&MCPServer> {
        self.servers.get(name)
    }
    
    pub fn list_servers(&self) -> Vec<&MCPServer> {
        self.servers.values().collect()
    }
    
    pub fn get_enabled_servers(&self) -> Vec<&MCPServer> {
        self.servers.values().filter(|s| s.enabled).collect()
    }
    
    pub fn toggle_server(&mut self, name: &str, enabled: bool) -> bool {
        if let Some(server) = self.servers.get_mut(name) {
            server.enabled = enabled;
            true
        } else {
            false
        }
    }
    
    pub fn add_server(&mut self, server: MCPServer) {
        self.servers.insert(server.name.clone(), server);
    }
    
    pub fn remove_server(&mut self, name: &str) -> Option<MCPServer> {
        self.servers.remove(name)
    }
}