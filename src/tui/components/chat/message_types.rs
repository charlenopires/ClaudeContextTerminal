//! Enhanced message types for the chat system
//!
//! This module defines comprehensive message types that support rich content,
//! tool calls, attachments, and streaming updates.

use crate::llm::types::{ContentBlock, MessageRole, ToolCall};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Enhanced message type for chat interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub attachments: Vec<MessageAttachment>,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<ToolResult>,
    pub streaming_state: StreamingState,
    pub finish_reason: Option<FinishReason>,
    pub thinking_content: Option<String>,
    pub reasoning_duration: Option<std::time::Duration>,
}

/// Attachment to a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub data: Vec<u8>,
    pub url: Option<String>,
}

/// Result from a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub error: Option<String>,
    pub execution_time: Option<std::time::Duration>,
    pub artifacts: Vec<ToolArtifact>,
}

/// Artifact created by a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolArtifact {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub data: Vec<u8>,
}

/// Streaming state of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamingState {
    /// Message is not being streamed
    Complete,
    /// Message is currently being streamed
    Streaming,
    /// Message streaming has been cancelled
    Cancelled,
    /// Message streaming failed
    Failed(String),
}

/// Reason why message generation finished
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Generation completed normally
    Stop,
    /// Hit maximum token limit
    Length,
    /// Content was filtered
    ContentFilter,
    /// Model requested tool use
    ToolCalls,
    /// Generation was cancelled
    Cancelled,
    /// An error occurred
    Error(String),
}

/// Message display options
#[derive(Debug, Clone)]
pub struct MessageDisplayOptions {
    pub show_timestamps: bool,
    pub show_metadata: bool,
    pub show_thinking: bool,
    pub compact_mode: bool,
    pub syntax_highlighting: bool,
    pub markdown_rendering: bool,
    pub word_wrap: bool,
    pub max_width: Option<usize>,
}

impl Default for MessageDisplayOptions {
    fn default() -> Self {
        Self {
            show_timestamps: true,
            show_metadata: false,
            show_thinking: true,
            compact_mode: false,
            syntax_highlighting: true,
            markdown_rendering: true,
            word_wrap: true,
            max_width: None,
        }
    }
}

impl ChatMessage {
    /// Create a new chat message
    pub fn new(role: MessageRole, content: Vec<ContentBlock>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            attachments: Vec::new(),
            tool_calls: Vec::new(),
            tool_results: Vec::new(),
            streaming_state: StreamingState::Complete,
            finish_reason: None,
            thinking_content: None,
            reasoning_duration: None,
        }
    }

    /// Create a new user message with text content
    pub fn new_user_text(text: String) -> Self {
        Self::new(
            MessageRole::User,
            vec![ContentBlock::Text { text }],
        )
    }

    /// Create a new assistant message with text content
    pub fn new_assistant_text(text: String) -> Self {
        Self::new(
            MessageRole::Assistant,
            vec![ContentBlock::Text { text }],
        )
    }

    /// Create a new system message with text content
    pub fn new_system_text(text: String) -> Self {
        Self::new(
            MessageRole::System,
            vec![ContentBlock::Text { text }],
        )
    }

    /// Get text content from all text blocks
    pub fn get_text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Add an attachment to the message
    pub fn add_attachment(&mut self, attachment: MessageAttachment) {
        self.attachments.push(attachment);
    }

    /// Add a tool call to the message
    pub fn add_tool_call(&mut self, tool_call: ToolCall) {
        self.tool_calls.push(tool_call);
    }

    /// Add a tool result to the message
    pub fn add_tool_result(&mut self, tool_result: ToolResult) {
        self.tool_results.push(tool_result);
    }

    /// Check if message is currently being streamed
    pub fn is_streaming(&self) -> bool {
        self.streaming_state == StreamingState::Streaming
    }

    /// Check if message is complete
    pub fn is_complete(&self) -> bool {
        self.streaming_state == StreamingState::Complete
    }

    /// Check if message has attachments
    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }

    /// Check if message has tool calls
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// Check if message has tool results
    pub fn has_tool_results(&self) -> bool {
        !self.tool_results.is_empty()
    }

    /// Check if message has thinking content
    pub fn has_thinking_content(&self) -> bool {
        self.thinking_content.is_some()
    }

    /// Set streaming state
    pub fn set_streaming_state(&mut self, state: StreamingState) {
        self.streaming_state = state;
    }

    /// Set finish reason
    pub fn set_finish_reason(&mut self, reason: FinishReason) {
        self.finish_reason = Some(reason);
    }

    /// Set thinking content
    pub fn set_thinking_content(&mut self, content: String) {
        self.thinking_content = Some(content);
    }

    /// Update message content (used during streaming)
    pub fn update_content(&mut self, new_text: String) {
        // Find the first text block and update it, or create one if none exists
        for block in &mut self.content {
            if let ContentBlock::Text { text } = block {
                *text = new_text;
                return;
            }
        }
        
        // No text block found, add one
        self.content.push(ContentBlock::Text { text: new_text });
    }

    /// Append text to existing content (used during streaming)
    pub fn append_content(&mut self, additional_text: String) {
        // Find the first text block and append to it, or create one if none exists
        for block in &mut self.content {
            if let ContentBlock::Text { text } = block {
                text.push_str(&additional_text);
                return;
            }
        }
        
        // No text block found, add one
        self.content.push(ContentBlock::Text { text: additional_text });
    }

    /// Get the total character count of all text content
    pub fn character_count(&self) -> usize {
        self.get_text_content().chars().count()
    }

    /// Get the number of lines in the text content
    pub fn line_count(&self) -> usize {
        self.get_text_content().lines().count()
    }

    /// Check if the message is from a specific role
    pub fn is_from_role(&self, role: &MessageRole) -> bool {
        &self.role == role
    }

    /// Get message age
    pub fn age(&self) -> chrono::Duration {
        Utc::now() - self.timestamp
    }

    /// Check if message contains code blocks
    pub fn has_code_blocks(&self) -> bool {
        let text = self.get_text_content();
        text.contains("```") || text.contains("`")
    }

    /// Extract code blocks from message content
    pub fn extract_code_blocks(&self) -> Vec<CodeBlock> {
        let text = self.get_text_content();
        let mut code_blocks = Vec::new();
        let mut in_code_block = false;
        let mut current_language = None;
        let mut current_code = String::new();
        
        for line in text.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    code_blocks.push(CodeBlock {
                        language: current_language.take(),
                        code: current_code.clone(),
                    });
                    current_code.clear();
                    in_code_block = false;
                } else {
                    // Start of code block
                    let language = line.trim_start_matches("```").trim();
                    current_language = if language.is_empty() {
                        None
                    } else {
                        Some(language.to_string())
                    };
                    in_code_block = true;
                }
            } else if in_code_block {
                if !current_code.is_empty() {
                    current_code.push('\n');
                }
                current_code.push_str(line);
            }
        }
        
        code_blocks
    }
}

/// Extracted code block from message content
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub code: String,
}

impl MessageAttachment {
    /// Create a new attachment
    pub fn new(filename: String, content_type: String, data: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            filename,
            content_type,
            size: data.len() as u64,
            data,
            url: None,
        }
    }

    /// Create an attachment from a file path
    pub fn from_file_path(file_path: &str) -> Result<Self, std::io::Error> {
        use std::fs;
        use std::path::Path;

        let path = Path::new(file_path);
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let data = fs::read(file_path)?;
        
        // Detect content type based on file extension
        let content_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("txt") => "text/plain",
            Some("md") => "text/markdown",
            Some("json") => "application/json",
            Some("xml") => "application/xml",
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "text/javascript",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("pdf") => "application/pdf",
            _ => "application/octet-stream",
        }.to_string();

        Ok(Self::new(filename, content_type, data))
    }

    /// Check if attachment is an image
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if attachment is text
    pub fn is_text(&self) -> bool {
        self.content_type.starts_with("text/") || 
        self.content_type == "application/json" ||
        self.content_type == "application/xml"
    }

    /// Get human-readable file size
    pub fn formatted_size(&self) -> String {
        let size = self.size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

impl ToolResult {
    /// Create a new tool result
    pub fn new(tool_call_id: String, content: String) -> Self {
        Self {
            tool_call_id,
            content,
            error: None,
            execution_time: None,
            artifacts: Vec::new(),
        }
    }

    /// Create a tool result with an error
    pub fn with_error(tool_call_id: String, error: String) -> Self {
        Self {
            tool_call_id,
            content: String::new(),
            error: Some(error),
            execution_time: None,
            artifacts: Vec::new(),
        }
    }

    /// Check if the tool result represents an error
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Add an artifact to the result
    pub fn add_artifact(&mut self, artifact: ToolArtifact) {
        self.artifacts.push(artifact);
    }
}

impl Default for StreamingState {
    fn default() -> Self {
        Self::Complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let message = ChatMessage::new_user_text("Hello, world!".to_string());
        
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.get_text_content(), "Hello, world!");
        assert!(!message.is_streaming());
        assert!(message.is_complete());
    }

    #[test]
    fn test_message_streaming_state() {
        let mut message = ChatMessage::new_assistant_text("".to_string());
        
        message.set_streaming_state(StreamingState::Streaming);
        assert!(message.is_streaming());
        assert!(!message.is_complete());
        
        message.append_content("Hello".to_string());
        message.append_content(" world!".to_string());
        assert_eq!(message.get_text_content(), "Hello world!");
        
        message.set_streaming_state(StreamingState::Complete);
        assert!(!message.is_streaming());
        assert!(message.is_complete());
    }

    #[test]
    fn test_code_block_extraction() {
        let content = r#"Here's some code:

```rust
fn main() {
    println!("Hello, world!");
}
```

And some more text."#;
        
        let message = ChatMessage::new_assistant_text(content.to_string());
        let code_blocks = message.extract_code_blocks();
        
        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0].language, Some("rust".to_string()));
        assert!(code_blocks[0].code.contains("fn main()"));
    }

    #[test]
    fn test_attachment_creation() {
        let data = b"Hello, world!".to_vec();
        let attachment = MessageAttachment::new(
            "test.txt".to_string(),
            "text/plain".to_string(),
            data.clone(),
        );
        
        assert_eq!(attachment.filename, "test.txt");
        assert_eq!(attachment.size, data.len() as u64);
        assert!(attachment.is_text());
        assert!(!attachment.is_image());
    }

    #[test]
    fn test_tool_result() {
        let result = ToolResult::new(
            "tool_call_123".to_string(),
            "Tool executed successfully".to_string(),
        );
        
        assert!(!result.is_error());
        assert_eq!(result.tool_call_id, "tool_call_123");
        
        let error_result = ToolResult::with_error(
            "tool_call_456".to_string(),
            "Tool execution failed".to_string(),
        );
        
        assert!(error_result.is_error());
    }
}