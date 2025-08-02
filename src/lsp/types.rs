//! LSP types and data structures

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;
use std::path::PathBuf;

/// LSP client configuration for a specific language
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LspClientConfig {
    /// Command to start the language server
    pub command: String,
    /// Arguments to pass to the language server
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the language server
    pub working_dir: Option<PathBuf>,
    /// Whether to enable workspace features
    #[serde(default = "default_workspace")]
    pub workspace: bool,
    /// File extensions this server handles
    #[serde(default)]
    pub file_extensions: Vec<String>,
}

fn default_workspace() -> bool {
    true
}

/// LSP configuration for all languages
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct LspConfig {
    /// Language server configurations by language ID
    #[serde(default)]
    pub servers: HashMap<String, LspClientConfig>,
    /// Global LSP settings
    #[serde(default)]
    pub settings: LspSettings,
}

/// Global LSP settings
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LspSettings {
    /// Whether LSP is enabled globally
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Timeout for LSP operations in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
    /// Maximum number of diagnostics to keep per file
    #[serde(default = "default_max_diagnostics")]
    pub max_diagnostics: usize,
}

fn default_enabled() -> bool {
    true
}

fn default_timeout() -> u64 {
    5000 // 5 seconds
}

fn default_max_diagnostics() -> usize {
    100
}

impl Default for LspSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            timeout_ms: default_timeout(),
            max_diagnostics: default_max_diagnostics(),
        }
    }
}

/// Information about an open file in LSP
#[derive(Debug, Clone)]
pub struct OpenFileInfo {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub content: String,
}

/// LSP diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// LSP diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub severity: Option<DiagnosticSeverity>,
    pub line: u32,
    pub character: u32,
    pub end_line: Option<u32>,
    pub end_character: Option<u32>,
    pub source: Option<String>,
    pub code: Option<String>,
}

/// LSP server capabilities
#[derive(Debug, Clone, Default)]
pub struct ServerCapabilities {
    pub text_document_sync: bool,
    pub hover: bool,
    pub completion: bool,
    pub signature_help: bool,
    pub goto_definition: bool,
    pub references: bool,
    pub document_highlight: bool,
    pub document_symbols: bool,
    pub workspace_symbols: bool,
    pub code_actions: bool,
    pub code_lens: bool,
    pub document_formatting: bool,
    pub document_range_formatting: bool,
    pub document_on_type_formatting: bool,
    pub rename: bool,
    pub document_link: bool,
    pub execute_command: bool,
    pub experimental: Option<serde_json::Value>,
}

/// LSP message types
#[derive(Debug, Clone)]
pub enum LspMessage {
    Request {
        id: i32,
        method: String,
        params: Option<serde_json::Value>,
    },
    Response {
        id: i32,
        result: Option<serde_json::Value>,
        error: Option<LspError>,
    },
    Notification {
        method: String,
        params: Option<serde_json::Value>,
    },
}

/// LSP error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Common LSP methods
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const SHUTDOWN: &str = "shutdown";
    pub const EXIT: &str = "exit";
    pub const TEXT_DOCUMENT_DID_OPEN: &str = "textDocument/didOpen";
    pub const TEXT_DOCUMENT_DID_CHANGE: &str = "textDocument/didChange";
    pub const TEXT_DOCUMENT_DID_CLOSE: &str = "textDocument/didClose";
    pub const TEXT_DOCUMENT_HOVER: &str = "textDocument/hover";
    pub const TEXT_DOCUMENT_COMPLETION: &str = "textDocument/completion";
    pub const TEXT_DOCUMENT_DEFINITION: &str = "textDocument/definition";
    pub const TEXT_DOCUMENT_REFERENCES: &str = "textDocument/references";
    pub const TEXT_DOCUMENT_DIAGNOSTICS: &str = "textDocument/publishDiagnostics";
}