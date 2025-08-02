//! File editing tool for making precise changes to files

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Tool for editing files with exact string replacements
pub struct EditTool;

impl EditTool {
    pub fn new() -> Self {
        Self
    }

    /// Perform exact string replacement in file content
    fn perform_edit(&self, content: &str, old_string: &str, new_string: &str, replace_all: bool) -> ToolResult<(String, usize)> {
        if old_string == new_string {
            return Err(anyhow::anyhow!("old_string and new_string cannot be the same"));
        }

        if old_string.is_empty() {
            return Err(anyhow::anyhow!("old_string cannot be empty"));
        }

        let replacement_count = if replace_all {
            content.matches(old_string).count()
        } else {
            if content.matches(old_string).count() != 1 {
                return Err(anyhow::anyhow!(
                    "old_string must appear exactly once in the file. Found {} occurrences. Use replace_all=true to replace all instances.",
                    content.matches(old_string).count()
                ));
            }
            1
        };

        if replacement_count == 0 {
            return Err(anyhow::anyhow!("old_string not found in file"));
        }

        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        Ok((new_content, replacement_count))
    }
}

#[async_trait]
impl BaseTool for EditTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let file_path = request.parameters.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        let old_string = request.parameters.get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: old_string"))?;

        let new_string = request.parameters.get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: new_string"))?;

        let replace_all = request.parameters.get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Security checks
        let path = Path::new(file_path);
        if !path.is_absolute() {
            return Err(anyhow::anyhow!("File path must be absolute"));
        }

        // Check for restricted paths
        for restricted in &request.permissions.restricted_paths {
            if file_path.starts_with(restricted) && !request.permissions.yolo_mode {
                return Err(anyhow::anyhow!("Access to path '{}' is restricted", file_path));
            }
        }

        if !request.permissions.allow_write && !request.permissions.yolo_mode {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some("Write permission required for file editing".to_string()),
            });
        }

        // Read current file content
        let current_content = match fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) => {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("Failed to read file '{}': {}", file_path, e)),
                });
            }
        };

        // Perform the edit
        match self.perform_edit(&current_content, old_string, new_string, replace_all) {
            Ok((new_content, replacement_count)) => {
                // Write the modified content back to the file
                match fs::write(&path, &new_content).await {
                    Ok(_) => {
                        let metadata = json!({
                            "file_path": file_path,
                            "old_string": old_string,
                            "new_string": new_string,
                            "replace_all": replace_all,
                            "replacements_made": replacement_count,
                            "original_size": current_content.len(),
                            "new_size": new_content.len(),
                        });

                        Ok(ToolResponse {
                            content: format!(
                                "Successfully edited file '{}'. Made {} replacement(s).",
                                file_path, replacement_count
                            ),
                            success: true,
                            metadata: Some(metadata),
                            error: None,
                        })
                    }
                    Err(e) => Ok(ToolResponse {
                        content: String::new(),
                        success: false,
                        metadata: None,
                        error: Some(format!("Failed to write file '{}': {}", file_path, e)),
                    })
                }
            }
            Err(e) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: Some(json!({
                    "file_path": file_path,
                    "old_string": old_string,
                    "new_string": new_string,
                })),
                error: Some(e.to_string()),
            })
        }
    }

    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Perform exact string replacements in files. The edit will FAIL if old_string is not unique unless replace_all is true."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to modify"
                },
                "old_string": {
                    "type": "string",
                    "description": "The text to replace"
                },
                "new_string": {
                    "type": "string", 
                    "description": "The text to replace it with (must be different from old_string)"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences of old_string (default false)",
                    "default": false
                }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    fn requires_permission(&self) -> bool {
        true // File editing requires write permission
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use crate::llm::tools::{ToolPermissions, ToolRequest};

    #[tokio::test]
    async fn test_simple_edit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let original_content = "Hello world\nThis is a test\nHello again";
        temp_file.write_all(original_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        let tool = EditTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_file.path().to_str().unwrap()));
        params.insert("old_string".to_string(), json!("This is a test"));
        params.insert("new_string".to_string(), json!("This is modified"));
        
        let mut permissions = ToolPermissions::default();
        permissions.allow_write = true;
        
        let request = ToolRequest {
            tool_name: "edit".to_string(),
            parameters: params,
            working_directory: None,
            permissions,
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        
        // Verify the file was actually modified
        let new_content = tokio::fs::read_to_string(temp_file.path()).await.unwrap();
        assert!(new_content.contains("This is modified"));
        assert!(!new_content.contains("This is a test"));
    }

    #[tokio::test]
    async fn test_replace_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let original_content = "Hello world\nHello everyone\nHello again";
        temp_file.write_all(original_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        let tool = EditTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_file.path().to_str().unwrap()));
        params.insert("old_string".to_string(), json!("Hello"));
        params.insert("new_string".to_string(), json!("Hi"));
        params.insert("replace_all".to_string(), json!(true));
        
        let mut permissions = ToolPermissions::default();
        permissions.allow_write = true;
        
        let request = ToolRequest {
            tool_name: "edit".to_string(),
            parameters: params,
            working_directory: None,
            permissions,
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        
        // Verify all instances were replaced
        let new_content = tokio::fs::read_to_string(temp_file.path()).await.unwrap();
        assert_eq!(new_content.matches("Hi").count(), 3);
        assert_eq!(new_content.matches("Hello").count(), 0);
    }

    #[tokio::test]
    async fn test_non_unique_string_without_replace_all() {
        let tool = EditTool::new();
        let content = "Hello world\nHello everyone";
        
        let result = tool.perform_edit(content, "Hello", "Hi", false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly once"));
    }

    #[tokio::test]
    async fn test_string_not_found() {
        let tool = EditTool::new();
        let content = "Hello world";
        
        let result = tool.perform_edit(content, "Goodbye", "Hi", false);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Found 0 occurrences"));
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let tool = EditTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!("/tmp/test.txt"));
        params.insert("old_string".to_string(), json!("old"));
        params.insert("new_string".to_string(), json!("new"));
        
        let permissions = ToolPermissions::default(); // write = false by default
        
        let request = ToolRequest {
            tool_name: "edit".to_string(),
            parameters: params,
            working_directory: None,
            permissions,
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.unwrap().contains("Write permission required"));
    }
}