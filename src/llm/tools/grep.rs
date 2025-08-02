//! Text search tool using grep-like functionality

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use regex::Regex;
use serde_json::json;
use std::path::Path;
use tokio::fs;

/// Tool for searching text in files
pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self {
        Self
    }

    /// Search for pattern in content
    async fn search_content(&self, content: &str, pattern: &str, case_insensitive: bool, line_numbers: bool, context_before: usize, context_after: usize) -> ToolResult<Vec<String>> {
        let regex = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))
        } else {
            Regex::new(pattern)
        }.map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

        let lines: Vec<&str> = content.lines().collect();
        let mut results = Vec::new();
        let mut matched_lines = Vec::new();

        // Find all matching lines
        for (line_num, line) in lines.iter().enumerate() {
            if regex.is_match(line) {
                matched_lines.push(line_num);
            }
        }

        // Collect results with context
        let mut processed_lines = std::collections::HashSet::new();
        
        for &match_line in &matched_lines {
            let start = match_line.saturating_sub(context_before);
            let end = (match_line + context_after + 1).min(lines.len());
            
            for i in start..end {
                if processed_lines.contains(&i) {
                    continue;
                }
                processed_lines.insert(i);
                
                let line_content = lines[i];
                let formatted_line = if line_numbers {
                    if i == match_line {
                        format!("{:4}:{}", i + 1, line_content)
                    } else {
                        format!("{:4}-{}", i + 1, line_content)
                    }
                } else {
                    line_content.to_string()
                };
                
                results.push((i, formatted_line));
            }
        }

        // Sort by line number and extract formatted content
        results.sort_by_key(|(line_num, _)| *line_num);
        Ok(results.into_iter().map(|(_, content)| content).collect())
    }
}

#[async_trait]
impl BaseTool for GrepTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let pattern = request.parameters.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;

        let file_path = request.parameters.get("path")
            .and_then(|v| v.as_str());

        let case_insensitive = request.parameters.get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let line_numbers = request.parameters.get("line_numbers")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let context_before = request.parameters.get("context_before")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let context_after = request.parameters.get("context_after")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Read file content
        let content = if let Some(path_str) = file_path {
            let path = Path::new(path_str);
            if !path.is_absolute() {
                return Err(anyhow::anyhow!("File path must be absolute"));
            }

            // Check for restricted paths
            for restricted in &request.permissions.restricted_paths {
                if path_str.starts_with(restricted) && !request.permissions.yolo_mode {
                    return Err(anyhow::anyhow!("Access to path '{}' is restricted", path_str));
                }
            }

            match fs::read_to_string(&path).await {
                Ok(content) => content,
                Err(e) => {
                    return Ok(ToolResponse {
                        content: String::new(),
                        success: false,
                        metadata: None,
                        error: Some(format!("Failed to read file '{}': {}", path_str, e)),
                    });
                }
            }
        } else {
            // If no file path provided, expect content in parameters
            request.parameters.get("content")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Either 'path' or 'content' parameter is required"))?
                .to_string()
        };

        // Perform search
        match self.search_content(&content, pattern, case_insensitive, line_numbers, context_before, context_after).await {
            Ok(matches) => {
                let result_content = if matches.is_empty() {
                    "No matches found.".to_string()
                } else {
                    matches.join("\n")
                };

                let metadata = json!({
                    "pattern": pattern,
                    "file_path": file_path,
                    "case_insensitive": case_insensitive,
                    "line_numbers": line_numbers,
                    "context_before": context_before,
                    "context_after": context_after,
                    "matches_found": matches.len(),
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
                metadata: Some(json!({
                    "pattern": pattern,
                    "file_path": file_path,
                })),
                error: Some(e.to_string()),
            })
        }
    }

    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search for text patterns in files or content using regular expressions. Supports context lines and case-insensitive search."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regular expression pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "The absolute path to the file to search (optional if content is provided)"
                },
                "content": {
                    "type": "string",
                    "description": "Text content to search (optional if path is provided)"
                },
                "case_insensitive": {
                    "type": "boolean",
                    "description": "Perform case-insensitive search",
                    "default": false
                },
                "line_numbers": {
                    "type": "boolean",
                    "description": "Show line numbers in output",
                    "default": true
                },
                "context_before": {
                    "type": "integer",
                    "description": "Number of lines to show before each match",
                    "default": 0
                },
                "context_after": {
                    "type": "integer",
                    "description": "Number of lines to show after each match",
                    "default": 0
                }
            },
            "required": ["pattern"]
        })
    }

    fn requires_permission(&self) -> bool {
        false // Text search is generally safe
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
    async fn test_grep_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Line 1\nThis is a test\nLine 3\nAnother test line\nLine 5";
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        
        let tool = GrepTool::new();
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!("test"));
        params.insert("path".to_string(), json!(temp_file.path().to_str().unwrap()));
        
        let request = ToolRequest {
            tool_name: "grep".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("This is a test"));
        assert!(response.content.contains("Another test line"));
    }

    #[tokio::test]
    async fn test_grep_content() {
        let tool = GrepTool::new();
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!("hello"));
        params.insert("content".to_string(), json!("Hello World\nhello world\nGoodbye"));
        params.insert("case_insensitive".to_string(), json!(true));
        
        let request = ToolRequest {
            tool_name: "grep".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("Hello World"));
        assert!(response.content.contains("hello world"));
        assert!(!response.content.contains("Goodbye"));
    }

    #[tokio::test]
    async fn test_grep_with_context() {
        let tool = GrepTool::new();
        let content = "Line 1\nLine 2\nMatch here\nLine 4\nLine 5";
        
        let result = tool.search_content(content, "Match", false, true, 1, 1).await.unwrap();
        
        assert_eq!(result.len(), 3); // Should include 1 before + match + 1 after
        assert!(result.iter().any(|line| line.contains("Line 2")));
        assert!(result.iter().any(|line| line.contains("Match here")));
        assert!(result.iter().any(|line| line.contains("Line 4")));
    }

    #[tokio::test]
    async fn test_grep_no_matches() {
        let tool = GrepTool::new();
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), json!("nonexistent"));
        params.insert("content".to_string(), json!("Hello World\nGoodbye"));
        
        let request = ToolRequest {
            tool_name: "grep".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions::default(),
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(response.success);
        assert!(response.content.contains("No matches found"));
    }

    #[tokio::test]
    async fn test_invalid_regex() {
        let tool = GrepTool::new();
        let content = "test content";
        
        let result = tool.search_content(content, "[invalid", false, true, 0, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid regex pattern"));
    }
}