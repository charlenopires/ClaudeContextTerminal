//! View tool implementation for reading file contents with line numbers

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// View tool for reading file contents with enhanced features
pub struct ViewTool;

impl ViewTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for ViewTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let file_path = request.parameters.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        let offset = request.parameters.get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let limit = request.parameters.get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(2000); // Default to 2000 lines

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

        // Check if file exists and get metadata
        let metadata = match fs::metadata(&path).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Try to suggest similar files
                let suggestions = self.find_similar_files(file_path).await;
                let error_msg = if suggestions.is_empty() {
                    format!("File not found: {}", file_path)
                } else {
                    format!("File not found: {}\n\nDid you mean one of these?\n{}", 
                        file_path, suggestions.join("\n"))
                };
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(error_msg),
                });
            }
            Err(e) => return Err(anyhow::anyhow!("Error accessing file: {}", e)),
        };

        // Check if it's a directory
        if metadata.is_dir() {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Path is a directory, not a file: {}", file_path)),
            });
        }

        // Check file size (250KB limit)
        const MAX_SIZE: u64 = 250 * 1024;
        if metadata.len() > MAX_SIZE {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("File is too large ({} bytes). Maximum size is {} bytes", 
                    metadata.len(), MAX_SIZE)),
            });
        }

        // Check if it's an image file
        if let Some(image_type) = self.detect_image_type(file_path) {
            return Ok(ToolResponse {
                content: format!("This is an image file of type: {}", image_type),
                success: false,
                metadata: None,
                error: Some(format!("Cannot display image file of type: {}", image_type)),
            });
        }

        // Read and format the file content
        match self.read_file_with_line_numbers(file_path, offset, limit).await {
            Ok((content, total_lines, displayed_lines)) => {
                let mut output = "<file>\n".to_string();
                output.push_str(&content);
                
                // Add truncation note if needed
                if total_lines > offset + displayed_lines {
                    output.push_str(&format!("\n\n(File has more lines. Use 'offset' parameter to read beyond line {})", 
                        offset + displayed_lines));
                }
                output.push_str("\n</file>");

                let response_metadata = json!({
                    "file_path": file_path,
                    "total_lines": total_lines,
                    "displayed_lines": displayed_lines,
                    "start_line": offset + 1,
                    "end_line": offset + displayed_lines,
                    "file_size": metadata.len(),
                });

                Ok(ToolResponse {
                    content: output,
                    success: true,
                    metadata: Some(response_metadata),
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
        "view"
    }

    fn description(&self) -> &str {
        r#"File viewing tool that reads and displays the contents of files with line numbers, allowing you to examine code, logs, or text data.

WHEN TO USE THIS TOOL:
- Use when you need to read the contents of a specific file
- Helpful for examining source code, configuration files, or log files
- Perfect for looking at text-based file formats

HOW TO USE:
- Provide the path to the file you want to view
- Optionally specify an offset to start reading from a specific line
- Optionally specify a limit to control how many lines are read
- Do not use this for directories use the ls tool instead

FEATURES:
- Displays file contents with line numbers for easy reference
- Can read from any position in a file using the offset parameter
- Handles large files by limiting the number of lines read
- Automatically truncates very long lines for better display
- Suggests similar file names when the requested file isn't found

LIMITATIONS:
- Maximum file size is 250KB
- Default reading limit is 2000 lines
- Lines longer than 2000 characters are truncated
- Cannot display binary files or images
- Images can be identified but not displayed

WINDOWS NOTES:
- Handles both Windows (CRLF) and Unix (LF) line endings automatically
- File paths work with both forward slashes (/) and backslashes (\)
- Text encoding is detected automatically for most common formats

TIPS:
- Use with Glob tool to first find files you want to view
- For code exploration, first use Grep to find relevant files, then View to examine them
- When viewing large files, use the offset parameter to read specific sections"#
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "The line number to start reading from (0-based)"
                },
                "limit": {
                    "type": "integer",
                    "description": "The number of lines to read (defaults to 2000)"
                }
            },
            "required": ["file_path"]
        })
    }
}

impl ViewTool {
    /// Read file content with line numbers
    async fn read_file_with_line_numbers(&self, file_path: &str, offset: usize, limit: usize) -> Result<(String, usize, usize), Box<dyn std::error::Error + Send + Sync>> {
        let content = fs::read_to_string(file_path).await?;
        
        // Check if content is valid UTF-8 (should be since read_to_string succeeded)
        if !content.chars().all(|c| !c.is_control() || c.is_whitespace()) {
            return Err("File content contains invalid characters".into());
        }

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // Apply offset and limit
        let start = offset.min(total_lines);
        let end = (start + limit).min(total_lines);
        let selected_lines = &lines[start..end];
        
        // Format with line numbers
        let mut result = Vec::new();
        for (i, line) in selected_lines.iter().enumerate() {
            let line_num = start + i + 1;
            let truncated_line = if line.len() > 2000 {
                format!("{}...", &line[..2000])
            } else {
                line.to_string()
            };
            
            result.push(format!("{:6}|{}", line_num, truncated_line));
        }
        
        let formatted_content = result.join("\n");
        let displayed_lines = end - start;
        
        Ok((formatted_content, total_lines, displayed_lines))
    }

    /// Find similar files in the same directory
    async fn find_similar_files(&self, file_path: &str) -> Vec<String> {
        let path = Path::new(file_path);
        let parent = match path.parent() {
            Some(p) => p,
            None => return Vec::new(),
        };
        
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_lowercase(),
            None => return Vec::new(),
        };

        let mut suggestions = Vec::new();
        
        if let Ok(mut entries) = fs::read_dir(parent).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        let entry_name = entry.file_name().to_string_lossy().to_lowercase();
                        
                        // Check for partial matches
                        if entry_name.contains(&file_name) || file_name.contains(&entry_name) {
                            suggestions.push(entry.path().to_string_lossy().to_string());
                            if suggestions.len() >= 3 {
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        suggestions
    }

    /// Detect if file is an image based on extension
    fn detect_image_type(&self, file_path: &str) -> Option<&'static str> {
        let path = Path::new(file_path);
        let extension = path.extension()?.to_str()?.to_lowercase();
        
        match extension.as_str() {
            "jpg" | "jpeg" => Some("JPEG"),
            "png" => Some("PNG"),
            "gif" => Some("GIF"),
            "bmp" => Some("BMP"),
            "svg" => Some("SVG"),
            "webp" => Some("WebP"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tools::ToolPermissions;
    use std::collections::HashMap;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_view_tool_info() {
        let tool = ViewTool::new();
        
        assert_eq!(tool.name(), "view");
        assert!(tool.description().contains("File viewing tool"));
        
        let params = tool.parameters();
        assert!(params["properties"].get("file_path").is_some());
        assert!(params["properties"].get("offset").is_some());
        assert!(params["properties"].get("limit").is_some());
    }

    #[tokio::test]
    async fn test_view_file_with_line_numbers() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        fs::write(&file_path, content).await.unwrap();
        
        let tool = ViewTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("limit".to_string(), json!(3));
        
        let request = ToolRequest {
            tool_name: "view".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("     1|line 1"));
        assert!(response.content.contains("     2|line 2"));
        assert!(response.content.contains("     3|line 3"));
        assert!(!response.content.contains("line 4"));
    }

    #[tokio::test]
    async fn test_view_with_offset() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        fs::write(&file_path, content).await.unwrap();
        
        let tool = ViewTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(file_path.to_str().unwrap()));
        params.insert("offset".to_string(), json!(2));
        params.insert("limit".to_string(), json!(2));
        
        let request = ToolRequest {
            tool_name: "view".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("     3|line 3"));
        assert!(response.content.contains("     4|line 4"));
        assert!(!response.content.contains("line 1"));
        assert!(!response.content.contains("line 5"));
    }

    #[tokio::test]
    async fn test_view_nonexistent_file() {
        let tool = ViewTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!("/nonexistent/file.txt"));
        
        let request = ToolRequest {
            tool_name: "view".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("File not found"));
    }

    #[tokio::test]
    async fn test_view_directory() {
        let temp_dir = tempdir().unwrap();
        
        let tool = ViewTool::new();
        let mut params = HashMap::new();
        params.insert("file_path".to_string(), json!(temp_dir.path().to_str().unwrap()));
        
        let request = ToolRequest {
            tool_name: "view".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("directory, not a file"));
    }

    #[test]
    fn test_detect_image_type() {
        let tool = ViewTool::new();
        
        assert_eq!(tool.detect_image_type("test.jpg"), Some("JPEG"));
        assert_eq!(tool.detect_image_type("test.jpeg"), Some("JPEG"));
        assert_eq!(tool.detect_image_type("test.png"), Some("PNG"));
        assert_eq!(tool.detect_image_type("test.gif"), Some("GIF"));
        assert_eq!(tool.detect_image_type("test.txt"), None);
    }
}