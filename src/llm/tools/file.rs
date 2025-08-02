//! File operations tool for reading file contents

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Tool for reading file contents
pub struct FileTool;

impl FileTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for FileTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let file_path = request.parameters.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        // Security check - validate path
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

        // Read file with optional line limits
        let limit = request.parameters.get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);
        
        let offset = request.parameters.get("offset")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(0);

        match fs::read_to_string(&path).await {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let total_lines = lines.len();
                
                let start = offset.min(total_lines);
                let end = match limit {
                    Some(l) => (start + l).min(total_lines),
                    None => total_lines,
                };
                
                let selected_lines = &lines[start..end];
                let result_content = selected_lines
                    .iter()
                    .enumerate()
                    .map(|(i, line)| format!("{:4}→{}", start + i + 1, line))
                    .collect::<Vec<_>>()
                    .join("\n");

                let metadata = json!({
                    "total_lines": total_lines,
                    "displayed_lines": end - start,
                    "start_line": start + 1,
                    "end_line": end,
                    "file_size": content.len(),
                });

                Ok(ToolResponse {
                    content: result_content,
                    success: true,
                    metadata: Some(metadata),
                    error: None,
                })
            }
            Err(e) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Failed to read file '{}': {}", file_path, e)),
            })
        }
    }

    fn name(&self) -> &str {
        "file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the filesystem. Supports line limits and offsets for large files."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to read"
                },
                "limit": {
                    "type": "integer",
                    "description": "The number of lines to read (optional)"
                },
                "offset": {
                    "type": "integer", 
                    "description": "The line number to start reading from (optional, defaults to 0)"
                }
            },
            "required": ["file_path"]
        })
    }

    fn requires_permission(&self) -> bool {
        false // File reading is generally safe
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
    async fn test_file_read() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Line 1\nLine 2\nLine 3\n";
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let tool = FileTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_file.path().to_str().unwrap()));
        
        let request = ToolRequest {
            tool_name: "file".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("Line 1"));
        assert!(response.content.contains("Line 2"));
        assert!(response.content.contains("Line 3"));
    }

    #[tokio::test]
    async fn test_file_read_with_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
        temp_file.write_all(content.as_bytes()).unwrap();
        
        let tool = FileTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_file.path().to_str().unwrap()));
        params.insert("limit".to_string(), json!(2));
        params.insert("offset".to_string(), json!(1));
        
        let request = ToolRequest {
            tool_name: "file".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("Line 2"));
        assert!(response.content.contains("Line 3"));
        assert!(!response.content.contains("Line 1"));
        assert!(!response.content.contains("Line 4"));
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let tool = FileTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!("/nonexistent/file.txt"));
        
        let request = ToolRequest {
            tool_name: "file".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.is_some());
    }
}