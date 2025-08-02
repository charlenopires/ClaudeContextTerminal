//! Glob pattern matching tool

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Tool for finding files using glob patterns
pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for GlobTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let pattern = request.parameters.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: pattern"))?;

        // Basic glob implementation using walkdir
        // In a full implementation, this would use proper glob crate
        Ok(ToolResponse {
            content: format!("Glob pattern matching for '{}' - Not fully implemented yet", pattern),
            success: true,
            metadata: Some(json!({"pattern": pattern})),
            error: None,
        })
    }

    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Find files matching glob patterns. Currently a placeholder implementation."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match files against"
                }
            },
            "required": ["pattern"]
        })
    }
}