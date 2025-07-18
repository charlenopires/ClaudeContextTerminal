use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextFile {
    pub path: String,
    pub name: String,
    pub content: String,
    pub last_modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeMdContent {
    pub content: String,
    pub included_files: Vec<String>,
}

#[tauri::command]
pub async fn load_context(file_path: String) -> Result<String, String> {
    match std::fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("Failed to load context file: {}", e))
    }
}

#[tauri::command]
pub async fn save_context(file_path: String, content: String) -> Result<(), String> {
    match std::fs::write(&file_path, content) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to save context file: {}", e))
    }
}

#[tauri::command]
pub async fn list_md_files(directory: String) -> Result<Vec<ContextFile>, String> {
    let path = PathBuf::from(&directory);
    let mut md_files = Vec::new();

    if !path.exists() {
        return Ok(md_files);
    }

    match std::fs::read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_path = entry.path();
                    if let Some(extension) = file_path.extension() {
                        if extension == "md" {
                            if let Some(file_name) = file_path.file_name() {
                                if let Some(file_name_str) = file_name.to_str() {
                                    let content = std::fs::read_to_string(&file_path)
                                        .unwrap_or_else(|_| String::new());
                                    
                                    let metadata = std::fs::metadata(&file_path)
                                        .map_err(|e| format!("Failed to get metadata: {}", e))?;
                                    
                                    let last_modified = format!("{:?}", metadata.modified());

                                    md_files.push(ContextFile {
                                        path: file_path.to_string_lossy().to_string(),
                                        name: file_name_str.to_string(),
                                        content,
                                        last_modified,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read directory: {}", e))
    }

    Ok(md_files)
}

#[tauri::command]
pub async fn generate_claude_md(directory: String) -> Result<ClaudeMdContent, String> {
    let path = PathBuf::from(&directory);
    let claude_md_path = path.join("claude.md");
    
    // Check if claude.md already exists
    if claude_md_path.exists() {
        let content = std::fs::read_to_string(&claude_md_path)
            .map_err(|e| format!("Failed to read existing claude.md: {}", e))?;
        return Ok(ClaudeMdContent {
            content,
            included_files: vec!["claude.md".to_string()],
        });
    }

    // Generate basic structure
    let mut content = String::from("# Project Context\n\n");
    let mut included_files = Vec::new();

    // Look for common context files
    let context_files = [
        "requirements.md",
        "design.md", 
        "features.md",
        "structure.md",
        "tasklist.md"
    ];

    for file_name in &context_files {
        let file_path = path.join(file_name);
        if file_path.exists() {
            content.push_str(&format!("## {}\n", file_name.replace(".md", "").replace("_", " ").to_uppercase()));
            content.push_str(&format!("@include {}\n\n", file_name));
            included_files.push(file_name.to_string());
        } else {
            // Create template files
            let template_content = match *file_name {
                "requirements.md" => "# Requirements\n\n## Technologies\n- Technology stack to be defined\n\n## Dependencies\n- Dependencies to be listed\n",
                "design.md" => "# Design Specifications\n\n## UI/UX Guidelines\n- Design patterns to be defined\n\n## Architecture\n- System architecture to be documented\n",
                "features.md" => "# Features\n\n## Core Features\n- Core functionality to be listed\n\n## Future Features\n- Planned enhancements\n",
                "structure.md" => "# Project Structure\n\n## Directory Layout\n```\nproject/\n├── src/\n└── docs/\n```\n",
                "tasklist.md" => "# Task List\n\n## To Do\n- [ ] Task 1\n\n## In Progress\n- [ ] Task 2\n\n## Done\n- [x] Task 3\n",
                _ => ""
            };
            
            if let Err(e) = std::fs::write(&file_path, template_content) {
                eprintln!("Failed to create template file {}: {}", file_name, e);
            } else {
                content.push_str(&format!("## {}\n", file_name.replace(".md", "").replace("_", " ").to_uppercase()));
                content.push_str(&format!("@include {}\n\n", file_name));
                included_files.push(file_name.to_string());
            }
        }
    }

    // Save the generated claude.md
    if let Err(e) = std::fs::write(&claude_md_path, &content) {
        return Err(format!("Failed to create claude.md: {}", e));
    }

    Ok(ClaudeMdContent {
        content,
        included_files,
    })
}