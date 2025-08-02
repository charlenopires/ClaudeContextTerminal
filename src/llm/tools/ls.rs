//! Directory listing tool

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Tool for listing directory contents
pub struct LsTool;

impl LsTool {
    pub fn new() -> Self {
        Self
    }

    /// Check if path matches any of the ignore patterns
    fn should_ignore(&self, path: &str, ignore_patterns: &[String]) -> bool {
        ignore_patterns.iter().any(|pattern| {
            // Simple glob-like matching
            if pattern.ends_with("*") {
                let prefix = &pattern[..pattern.len() - 1];
                path.starts_with(prefix)
            } else if pattern.starts_with("*") {
                let suffix = &pattern[1..];
                path.ends_with(suffix)
            } else {
                path == pattern
            }
        })
    }
}

#[async_trait]
impl BaseTool for LsTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let path_str = request.parameters.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;

        let ignore_patterns: Vec<String> = request.parameters.get("ignore")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Security check - validate path
        let path = Path::new(path_str);
        if !path.is_absolute() {
            return Err(anyhow::anyhow!("Path must be absolute"));
        }

        // Check for restricted paths
        for restricted in &request.permissions.restricted_paths {
            if path_str.starts_with(restricted) && !request.permissions.yolo_mode {
                return Err(anyhow::anyhow!("Access to path '{}' is restricted", path_str));
            }
        }

        match fs::read_dir(&path).await {
            Ok(mut entries) => {
                let mut items = Vec::new();
                let mut directories = Vec::new();
                let mut files = Vec::new();

                while let Some(entry) = entries.next_entry().await.map_err(|e| {
                    anyhow::anyhow!("Error reading directory entry: {}", e)
                })? {
                    let entry_path = entry.path();
                    let name = entry_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("<invalid-name>")
                        .to_string();

                    // Skip if should be ignored
                    if self.should_ignore(&name, &ignore_patterns) {
                        continue;
                    }

                    let metadata = entry.metadata().await.map_err(|e| {
                        anyhow::anyhow!("Error reading metadata for '{}': {}", name, e)
                    })?;

                    if metadata.is_dir() {
                        directories.push(format!("    {}/", name));
                    } else {
                        files.push(format!("      {}", name));
                    }
                }

                // Sort directories and files separately
                directories.sort();
                files.sort();

                // Combine with header
                items.push(format!("- {}/", path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("<root>")));

                // Add directories first, then files
                items.extend(directories);
                items.extend(files);

                let content = items.join("\n");
                let total_items = items.len() - 1; // Subtract 1 for the header

                let metadata = json!({
                    "path": path_str,
                    "total_items": total_items,
                    "ignore_patterns": ignore_patterns,
                });

                Ok(ToolResponse {
                    content,
                    success: true,
                    metadata: Some(metadata),
                    error: None,
                })
            }
            Err(e) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Failed to read directory '{}': {}", path_str, e)),
            })
        }
    }

    fn name(&self) -> &str {
        "ls"
    }

    fn description(&self) -> &str {
        "List files and directories in a given path. Supports ignore patterns for filtering."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The absolute path to the directory to list"
                },
                "ignore": {
                    "type": "array",
                    "description": "List of glob patterns to ignore",
                    "items": {
                        "type": "string"
                    }
                }
            },
            "required": ["path"]
        })
    }

    fn requires_permission(&self) -> bool {
        false // Directory listing is generally safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use crate::llm::tools::{ToolPermissions, ToolRequest};

    #[tokio::test]
    async fn test_ls_directory() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create some test files and directories
        tokio::fs::create_dir(temp_path.join("subdir")).await.unwrap();
        tokio::fs::write(temp_path.join("file1.txt"), "content").await.unwrap();
        tokio::fs::write(temp_path.join("file2.rs"), "rust code").await.unwrap();
        
        let tool = LsTool::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!(temp_path.to_str().unwrap()));
        
        let request = ToolRequest {
            tool_name: "ls".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("subdir/"));
        assert!(response.content.contains("file1.txt"));
        assert!(response.content.contains("file2.rs"));
    }

    #[tokio::test]
    async fn test_ls_with_ignore_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // Create test files
        tokio::fs::write(temp_path.join("file1.txt"), "content").await.unwrap();
        tokio::fs::write(temp_path.join("file2.rs"), "rust code").await.unwrap();
        tokio::fs::write(temp_path.join("ignore_me.log"), "logs").await.unwrap();
        
        let tool = LsTool::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!(temp_path.to_str().unwrap()));
        params.insert("ignore".to_string(), json!(["*.log"]));
        
        let request = ToolRequest {
            tool_name: "ls".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("file1.txt"));
        assert!(response.content.contains("file2.rs"));
        assert!(!response.content.contains("ignore_me.log"));
    }

    #[tokio::test]
    async fn test_ls_nonexistent_directory() {
        let tool = LsTool::new();
        let mut params = HashMap::new();
        params.insert("path".to_string(), json!("/nonexistent/directory"));
        
        let request = ToolRequest {
            tool_name: "ls".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_ignore_pattern_matching() {
        let tool = LsTool::new();
        let patterns = vec!["*.log".to_string(), "temp*".to_string(), "exact_name".to_string()];
        
        assert!(tool.should_ignore("file.log", &patterns));
        assert!(tool.should_ignore("temp_file.txt", &patterns));
        assert!(tool.should_ignore("exact_name", &patterns));
        assert!(!tool.should_ignore("file.txt", &patterns));
        assert!(!tool.should_ignore("mytemp.txt", &patterns));
    }
}