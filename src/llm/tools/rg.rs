//! Ripgrep tool for fast text searching

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;

/// Tool for ripgrep-powered text search
pub struct RgTool;

impl RgTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl BaseTool for RgTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        // For now, delegate to grep tool since we don't have ripgrep binary integration yet
        // In a full implementation, this would execute the `rg` command
        let grep_tool = super::GrepTool::new();
        grep_tool.execute(request).await
    }

    fn name(&self) -> &str {
        "rg"
    }

    fn description(&self) -> &str {
        "Fast text search using ripgrep. Currently delegates to grep tool."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "The path to search in"
                }
            },
            "required": ["pattern"]
        })
    }
}