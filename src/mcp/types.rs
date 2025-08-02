//! MCP types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpTransportConfig {
    #[serde(rename = "stdio")]
    Stdio {
        command: String,
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
    #[serde(rename = "sse")]
    Sse {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name
    pub name: String,
    /// Transport configuration
    pub transport: McpTransportConfig,
    /// Server description
    pub description: Option<String>,
    /// Whether server is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Server initialization timeout
    #[serde(default = "default_init_timeout")]
    pub init_timeout_ms: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_init_timeout() -> u64 {
    10000 // 10 seconds
}

/// MCP configuration for all servers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    /// MCP servers by name
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
    /// Global MCP settings
    #[serde(default)]
    pub settings: McpSettings,
}

/// Global MCP settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSettings {
    /// Whether MCP is enabled globally
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Maximum number of concurrent MCP calls
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    /// Default timeout for MCP operations
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_max_concurrent() -> usize {
    10
}

fn default_timeout() -> u64 {
    30000 // 30 seconds
}

impl Default for McpSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_concurrent: default_max_concurrent(),
            timeout_ms: default_timeout(),
        }
    }
}

/// MCP protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub enum McpMessage {
    #[serde(rename = "2.0")]
    JsonRpc {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        method: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        params: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<McpError>,
    },
}

/// MCP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: McpToolSchema,
}

/// MCP tool schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// MCP tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// MCP tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<McpContent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// MCP content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        uri: String,
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
}

/// MCP resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// MCP prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPrompt {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<McpPromptArgument>>,
}

/// MCP prompt argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptArgument {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// MCP server capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<McpLoggingCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<McpPromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<McpResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<McpToolsCapability>,
}

/// MCP logging capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpLoggingCapability {}

/// MCP prompts capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPromptsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP resources capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResourcesCapability {
    #[serde(rename = "subscribe", skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP tools capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolsCapability {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// MCP client capabilities
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<serde_json::Value>,
}

/// Common MCP method names
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const PING: &str = "ping";
    pub const LIST_TOOLS: &str = "tools/list";
    pub const CALL_TOOL: &str = "tools/call";
    pub const LIST_RESOURCES: &str = "resources/list";
    pub const READ_RESOURCE: &str = "resources/read";
    pub const LIST_PROMPTS: &str = "prompts/list";
    pub const GET_PROMPT: &str = "prompts/get";
    pub const SET_LOGGING_LEVEL: &str = "logging/setLevel";
}

/// MCP error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}