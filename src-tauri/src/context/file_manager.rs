use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct FileManager;

impl FileManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn scan_markdown_files(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let mut md_files = Vec::new();
        
        if !directory.exists() || !directory.is_dir() {
            return Ok(md_files);
        }
        
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" {
                        md_files.push(path);
                    }
                }
            }
        }
        
        md_files.sort();
        Ok(md_files)
    }
    
    pub fn ensure_context_files(&self, directory: &Path) -> Result<Vec<PathBuf>> {
        let context_files = [
            "requirements.md",
            "design.md", 
            "features.md",
            "structure.md",
            "tasklist.md"
        ];
        
        let mut created_files = Vec::new();
        
        for file_name in &context_files {
            let file_path = directory.join(file_name);
            if !file_path.exists() {
                let template_content = self.get_template_content(file_name);
                std::fs::write(&file_path, template_content)?;
                created_files.push(file_path);
            }
        }
        
        Ok(created_files)
    }
    
    fn get_template_content(&self, file_name: &str) -> &'static str {
        match file_name {
            "requirements.md" => include_str!("../../../templates/requirements.md"),
            "design.md" => include_str!("../../../templates/design.md"),
            "features.md" => include_str!("../../../templates/features.md"),
            "structure.md" => include_str!("../../../templates/structure.md"),
            "tasklist.md" => include_str!("../../../templates/tasklist.md"),
            _ => "# Template\n\nContent to be added.\n",
        }
    }
}