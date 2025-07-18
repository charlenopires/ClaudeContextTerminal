use std::path::Path;
use anyhow::Result;

pub struct ContextParser;

impl ContextParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_claude_md(&self, file_path: &Path) -> Result<Vec<String>> {
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        let mut included_files = Vec::new();
        
        for line in content.lines() {
            if line.trim().starts_with("@include ") {
                if let Some(file_name) = line.strip_prefix("@include ") {
                    included_files.push(file_name.trim().to_string());
                }
            }
        }
        
        Ok(included_files)
    }
    
    pub fn resolve_includes(&self, content: &str, base_path: &Path) -> Result<String> {
        let mut resolved_content = String::new();
        
        for line in content.lines() {
            if line.trim().starts_with("@include ") {
                if let Some(file_name) = line.strip_prefix("@include ") {
                    let file_path = base_path.join(file_name.trim());
                    if file_path.exists() {
                        let included_content = std::fs::read_to_string(&file_path)?;
                        resolved_content.push_str(&included_content);
                        resolved_content.push('\n');
                    } else {
                        resolved_content.push_str(&format!("<!-- File not found: {} -->\n", file_name));
                    }
                }
            } else {
                resolved_content.push_str(line);
                resolved_content.push('\n');
            }
        }
        
        Ok(resolved_content)
    }
}