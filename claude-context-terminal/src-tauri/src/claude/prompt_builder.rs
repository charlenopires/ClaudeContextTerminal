use anyhow::Result;
use std::path::Path;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn new() -> Self {
        Self
    }
    
    pub fn build_contextual_prompt(
        &self,
        task_title: &str,
        task_description: &str,
        task_prompt: &str,
        context_files: &[String],
        mcp_servers: &[String],
    ) -> Result<String> {
        let mut prompt = String::new();
        
        // Add project context
        prompt.push_str("[CONTEXTO DO PROJETO]\n");
        for file_path in context_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let file_name = Path::new(file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(file_path);
                    
                prompt.push_str(&format!("## {}\n{}\n\n", file_name, content));
            }
        }
        
        // Add current task
        prompt.push_str("[TAREFA ATUAL]\n");
        prompt.push_str(&format!("Título: {}\n", task_title));
        prompt.push_str(&format!("Descrição: {}\n", task_description));
        
        // Add instructions
        prompt.push_str("\n[INSTRUÇÕES]\n");
        prompt.push_str(task_prompt);
        
        // Add available MCP servers
        if !mcp_servers.is_empty() {
            prompt.push_str("\n\n[SERVIDORES MCP DISPONÍVEIS]\n");
            for server in mcp_servers {
                prompt.push_str(&format!("- @{}\n", server));
            }
        }
        
        // Add productivity guidelines
        prompt.push_str("\n\n[DIRETRIZES]\n");
        prompt.push_str("- Use a abordagem mais eficiente possível\n");
        prompt.push_str("- Siga as convenções do projeto existente\n");
        prompt.push_str("- Escreva código limpo e bem documentado\n");
        prompt.push_str("- Execute testes quando aplicável\n");
        
        Ok(prompt)
    }
    
    pub fn build_mcp_enhanced_prompt(
        &self,
        base_prompt: &str,
        mcp_enhancements: &[String],
    ) -> String {
        let mut prompt = base_prompt.to_string();
        
        if !mcp_enhancements.is_empty() {
            prompt.push_str("\n\n[ENRIQUECIMENTOS MCP]\n");
            for enhancement in mcp_enhancements {
                prompt.push_str(&format!("- {}\n", enhancement));
            }
        }
        
        prompt
    }
}