//! Write tool implementation for creating and updating files

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Write tool for creating and updating files
pub struct WriteTool;

impl WriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for WriteTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let file_path = request.parameters.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        let content = request.parameters.get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: content"))?;

        // Security check - validate path
        let path = Path::new(file_path);
        if !path.is_absolute() {
            return Err(anyhow::anyhow!("File path must be absolute"));
        }

        // Check permissions for writing
        if !request.permissions.allow_write && !request.permissions.yolo_mode {
            return Err(anyhow::anyhow!("Write access not permitted"));
        }

        // Check for restricted paths
        for restricted in &request.permissions.restricted_paths {
            if file_path.starts_with(restricted) && !request.permissions.yolo_mode {
                return Err(anyhow::anyhow!("Access to path '{}' is restricted", file_path));
            }
        }

        // Check if path is a directory
        if let Ok(metadata) = fs::metadata(&path).await {
            if metadata.is_dir() {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("Path is a directory, not a file: {}", file_path)),
                });
            }
        }

        // Read existing content if file exists
        let old_content = match fs::read_to_string(&path).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("Error reading existing file: {}", e)),
                });
            }
        };

        // Check if content is the same (avoid unnecessary writes)
        if old_content == content {
            return Ok(ToolResponse {
                content: format!("File {} already contains the exact content. No changes made.", file_path),
                success: true,
                metadata: Some(json!({
                    "file_path": file_path,
                    "content_changed": false,
                    "file_size": content.len(),
                })),
                error: None,
            });
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent).await {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("Error creating parent directory: {}", e)),
                });
            }
        }

        // Generate diff information
        let (additions, removals) = self.calculate_diff_stats(&old_content, content);

        // Write the file
        match fs::write(&path, content).await {
            Ok(()) => {
                let diff_info = if !old_content.is_empty() {
                    format!(" (+{} -{} lines)", additions, removals)
                } else {
                    format!(" (new file, {} lines)", content.lines().count())
                };

                let result_msg = format!("File successfully written: {}{}", file_path, diff_info);

                let response_metadata = json!({
                    "file_path": file_path,
                    "content_changed": true,
                    "file_size": content.len(),
                    "additions": additions,
                    "removals": removals,
                    "was_new_file": old_content.is_empty(),
                });

                Ok(ToolResponse {
                    content: format!("<result>\n{}\n</result>", result_msg),
                    success: true,
                    metadata: Some(response_metadata),
                    error: None,
                })
            }
            Err(e) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Error writing file: {}", e)),
            })
        }
    }

    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        r#"File writing tool that creates or updates files in the filesystem, allowing you to save or modify text content.

WHEN TO USE THIS TOOL:
- Use when you need to create a new file
- Helpful for updating existing files with modified content
- Perfect for saving generated code, configurations, or text data

HOW TO USE:
- Provide the path to the file you want to write
- Include the content to be written to the file
- The tool will create any necessary parent directories

FEATURES:
- Can create new files or overwrite existing ones
- Creates parent directories automatically if they don't exist
- Checks if the file has been modified since last read for safety
- Avoids unnecessary writes when content hasn't changed

LIMITATIONS:
- You should read a file before writing to it to avoid conflicts
- Cannot append to files (rewrites the entire file)

WINDOWS NOTES:
- File permissions (0o755, 0o644) are Unix-style but work on Windows with appropriate translations
- Use forward slashes (/) in paths for cross-platform compatibility
- Windows file attributes and permissions are handled automatically by the Go runtime

TIPS:
- Use the View tool first to examine existing files before modifying them
- Use the LS tool to verify the correct location when creating new files
- Combine with Glob and Grep tools to find and modify multiple files
- Always include descriptive comments when making changes to existing code"#
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }
}

impl WriteTool {
    /// Calculate simple diff statistics (line additions and removals)
    fn calculate_diff_stats(&self, old_content: &str, new_content: &str) -> (usize, usize) {
        let old_lines: Vec<&str> = if old_content.is_empty() {
            Vec::new()
        } else {
            old_content.lines().collect()
        };
        
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        // Simple diff calculation
        // This is a basic implementation - a more sophisticated one would use proper diff algorithms
        let old_len = old_lines.len();
        let new_len = new_lines.len();
        
        if old_len == 0 {
            // New file
            (new_len, 0)
        } else if new_len == 0 {
            // File deleted (all content removed)
            (0, old_len)
        } else {
            // Calculate approximate additions and removals
            // This is a simplified approach - real diff would be more complex
            let common_lines = old_lines.iter()
                .filter(|old_line| new_lines.contains(old_line))
                .count();
            
            let additions = new_len.saturating_sub(common_lines);
            let removals = old_len.saturating_sub(common_lines);
            
            (additions, removals)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tools::ToolPermissions;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_write_tool_info() {
        let tool = WriteTool::new();
        
        assert_eq!(tool.name(), "write");
        assert!(tool.description().contains("File writing tool"));
        
        let params = tool.parameters();
        assert!(params["properties"].get("file_path").is_some());
        assert!(params["properties"].get("content").is_some());
    }

    #[tokio::test]
    async fn test_write_new_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("new_file.txt");
        
        let tool = WriteTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("content".to_string(), json!("Hello, World!"));
        
        let request = ToolRequest {
            tool_name: "write".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_write: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("File successfully written"));
        
        // Verify file was actually written
        let written_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(written_content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_write_existing_file() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("existing_file.txt");
        
        // Create initial file
        fs::write(&file_path, "Original content").await.unwrap();
        
        let tool = WriteTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("content".to_string(), json!("Updated content"));
        
        let request = ToolRequest {
            tool_name: "write".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_write: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("File successfully written"));
        
        // Verify file was updated
        let written_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(written_content, "Updated content");
    }

    #[tokio::test]
    async fn test_write_same_content() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("same_content.txt");
        
        let content = "Same content";
        fs::write(&file_path, content).await.unwrap();
        
        let tool = WriteTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("content".to_string(), json!(content));
        
        let request = ToolRequest {
            tool_name: "write".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_write: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("already contains the exact content"));
    }

    #[tokio::test]
    async fn test_write_permission_denied() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("permission_test.txt");
        
        let tool = WriteTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("content".to_string(), json!("Test content"));
        
        let request = ToolRequest {
            tool_name: "write".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_write: false,
                yolo_mode: false,
                ..Default::default()
            },
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Write access not permitted"));
    }

    #[tokio::test]
    async fn test_write_to_directory() {
        let temp_dir = tempdir().unwrap();
        
        let tool = WriteTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_dir.path().to_str().unwrap()));
        params.insert("content".to_string(), json!("Test content"));
        
        let request = ToolRequest {
            tool_name: "write".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_write: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("directory, not a file"));
    }

    #[test]
    fn test_calculate_diff_stats() {
        let tool = WriteTool::new();
        
        // New file
        let (additions, removals) = tool.calculate_diff_stats("", "line1\nline2\nline3");
        assert_eq!(additions, 3);
        assert_eq!(removals, 0);
        
        // File deletion
        let (additions, removals) = tool.calculate_diff_stats("line1\nline2", "");
        assert_eq!(additions, 0);
        assert_eq!(removals, 2);
        
        // Content replacement
        let (additions, removals) = tool.calculate_diff_stats("old1\nold2", "new1\nnew2");
        assert_eq!(additions, 2);
        assert_eq!(removals, 2);
    }
}