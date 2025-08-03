//! Preview pane for completion items with detailed information

use super::CompletionItem;
use crate::tui::{
    components::{Component, ComponentState},
    themes::Theme,
    Frame,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame as RatatuiFrame,
};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

/// Preview component for displaying detailed completion information
pub struct CompletionPreview {
    state: ComponentState,
    current_item: Option<CompletionItem>,
    preview_content: String,
    preview_title: String,
    show_content_preview: bool,
    max_preview_lines: usize,
    show_metadata: bool,
}

impl CompletionPreview {
    /// Create a new completion preview
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(),
            current_item: None,
            preview_content: String::new(),
            preview_title: String::new(),
            show_content_preview: true,
            max_preview_lines: 20,
            show_metadata: true,
        }
    }

    /// Enable or disable content preview for files
    pub fn with_content_preview(mut self, enabled: bool) -> Self {
        self.show_content_preview = enabled;
        self
    }

    /// Set maximum number of preview lines
    pub fn with_max_preview_lines(mut self, max_lines: usize) -> Self {
        self.max_preview_lines = max_lines;
        self
    }

    /// Enable or disable metadata display
    pub fn with_metadata(mut self, show: bool) -> Self {
        self.show_metadata = show;
        self
    }

    /// Update preview with a new completion item
    pub async fn update_preview(&mut self, item: Option<CompletionItem>) -> Result<()> {
        self.current_item = item.clone();
        
        if let Some(ref item) = item {
            debug!("Updating preview for item: {}", item.title);
            self.preview_title = format!("{} [{}]", item.title, item.provider);
            self.preview_content = self.generate_preview_content(item).await;
        } else {
            self.preview_title.clear();
            self.preview_content.clear();
        }

        Ok(())
    }

    /// Generate preview content for the completion item
    async fn generate_preview_content(&self, item: &CompletionItem) -> String {
        let mut content = Vec::new();

        // Add basic information
        content.push(format!("Title: {}", item.title));
        content.push(format!("Value: {}", item.value));
        content.push(format!("Provider: {}", item.provider));
        content.push(format!("Score: {:.2}", item.score));

        if let Some(ref description) = item.description {
            content.push(format!("Description: {}", description));
        }

        content.push(String::new()); // Empty line

        // Add provider-specific content
        match item.provider.as_str() {
            "file" => {
                content.extend(self.generate_file_preview(&item.value).await);
            }
            "command" => {
                content.extend(self.generate_command_preview(&item.value).await);
            }
            "history" => {
                content.extend(self.generate_history_preview(item).await);
            }
            "code" => {
                content.extend(self.generate_code_preview(item).await);
            }
            _ => {
                content.push("No additional information available.".to_string());
            }
        }

        // Add metadata if available and enabled
        if self.show_metadata {
            if let Some(ref metadata) = item.metadata {
                content.push(String::new());
                content.push("Metadata:".to_string());
                if let Ok(pretty_json) = serde_json::to_string_pretty(metadata) {
                    content.push(pretty_json);
                } else {
                    content.push(metadata.to_string());
                }
            }
        }

        content.join("\n")
    }

    /// Generate preview content for file completions
    async fn generate_file_preview(&self, file_path: &str) -> Vec<String> {
        let mut content = Vec::new();
        let path = Path::new(file_path);

        // File information
        if path.exists() {
            if let Ok(metadata) = fs::metadata(path) {
                content.push(format!("Type: {}", if metadata.is_dir() { "Directory" } else { "File" }));
                content.push(format!("Size: {} bytes", metadata.len()));
                
                if let Ok(modified) = metadata.modified() {
                    if let Ok(duration) = modified.elapsed() {
                        let seconds = duration.as_secs();
                        let time_str = if seconds < 60 {
                            format!("{} seconds ago", seconds)
                        } else if seconds < 3600 {
                            format!("{} minutes ago", seconds / 60)
                        } else if seconds < 86400 {
                            format!("{} hours ago", seconds / 3600)
                        } else {
                            format!("{} days ago", seconds / 86400)
                        };
                        content.push(format!("Modified: {}", time_str));
                    }
                }
            }

            // File content preview for text files
            if self.show_content_preview && path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let is_text = matches!(ext.to_lowercase().as_str(), 
                        "txt" | "md" | "rs" | "py" | "js" | "ts" | "html" | "css" | 
                        "json" | "yaml" | "toml" | "xml" | "csv" | "log"
                    );

                    if is_text {
                        content.push(String::new());
                        content.push("Content Preview:".to_string());
                        content.push("─".repeat(40));

                        match fs::read_to_string(path) {
                            Ok(file_content) => {
                                let lines: Vec<&str> = file_content.lines().collect();
                                let preview_lines = lines.iter()
                                    .take(self.max_preview_lines)
                                    .map(|&line| {
                                        if line.len() > 80 {
                                            format!("{}...", &line[..77])
                                        } else {
                                            line.to_string()
                                        }
                                    })
                                    .collect::<Vec<_>>();

                                content.extend(preview_lines);

                                if lines.len() > self.max_preview_lines {
                                    content.push(format!("... ({} more lines)", lines.len() - self.max_preview_lines));
                                }
                            }
                            Err(e) => {
                                content.push(format!("Error reading file: {}", e));
                            }
                        }
                    }
                }
            }
        } else {
            content.push("File does not exist".to_string());
        }

        content
    }

    /// Generate preview content for command completions
    async fn generate_command_preview(&self, command: &str) -> Vec<String> {
        let mut content = Vec::new();

        // Command information
        content.push(format!("Command: {}", command));

        // Add common command descriptions
        let command_info = match command {
            "ls" => "List directory contents",
            "cd" => "Change directory",
            "pwd" => "Print working directory",
            "cat" => "Display file contents",
            "grep" => "Search text patterns",
            "find" => "Find files and directories",
            "git" => "Version control system",
            "cargo" => "Rust package manager",
            "npm" => "Node package manager",
            "docker" => "Container platform",
            "curl" => "Transfer data from servers",
            "vim" | "nvim" => "Text editor",
            "code" => "VS Code editor",
            "ssh" => "Secure shell connection",
            "ps" => "List running processes",
            "top" => "Display running processes",
            "kill" => "Terminate processes",
            _ => "System command",
        };

        content.push(format!("Type: {}", command_info));

        // Check if command exists in PATH
        if let Ok(path_var) = std::env::var("PATH") {
            let found = path_var.split(':').any(|dir| {
                Path::new(dir).join(command).exists()
            });
            content.push(format!("Available: {}", if found { "Yes" } else { "No" }));
        }

        // Add usage examples for common commands
        let usage_example = match command {
            "ls" => Some(vec!["ls -la", "ls -lh", "ls *.txt"]),
            "cd" => Some(vec!["cd ~/Documents", "cd ..", "cd /path/to/dir"]),
            "grep" => Some(vec!["grep 'pattern' file.txt", "grep -r 'pattern' .", "grep -i 'pattern' *.log"]),
            "find" => Some(vec!["find . -name '*.rs'", "find /path -type f", "find . -mtime -7"]),
            "git" => Some(vec!["git status", "git commit -m 'message'", "git push origin main"]),
            "cargo" => Some(vec!["cargo build", "cargo test", "cargo run"]),
            _ => None,
        };

        if let Some(examples) = usage_example {
            content.push(String::new());
            content.push("Usage Examples:".to_string());
            for example in examples {
                content.push(format!("  {}", example));
            }
        }

        content
    }

    /// Generate preview content for history completions
    async fn generate_history_preview(&self, item: &CompletionItem) -> Vec<String> {
        let mut content = Vec::new();

        // Extract usage frequency from description if available
        if let Some(ref description) = item.description {
            content.push(format!("Usage: {}", description));
        }

        // Analyze the completion type
        let completion_type = if item.value.contains('/') || item.value.contains('\\') {
            "File path"
        } else if item.value.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            "Command or identifier"
        } else {
            "Text phrase"
        };

        content.push(format!("Type: {}", completion_type));

        // Add context-based suggestions
        if completion_type == "Command or identifier" {
            content.push(String::new());
            content.push("Similar patterns you've used:".to_string());
            // This would be enhanced with actual historical data
            content.push("  (Historical patterns would be shown here)".to_string());
        }

        content
    }

    /// Generate preview content for code completions
    async fn generate_code_preview(&self, item: &CompletionItem) -> Vec<String> {
        let mut content = Vec::new();

        // Determine completion type
        let completion_type = if item.title.ends_with('(') || item.title.contains("()") {
            "Function/Method"
        } else if item.title.chars().all(|c| c.is_lowercase() || c == '_') {
            "Keyword"
        } else if item.title.starts_with(char::is_uppercase) {
            "Type/Class"
        } else {
            "Identifier"
        };

        content.push(format!("Type: {}", completion_type));

        // Add language-specific information
        match completion_type {
            "Function/Method" => {
                content.push("Signature: (parameters would be shown here)".to_string());
                content.push("Documentation: (doc comments would be shown here)".to_string());
            }
            "Keyword" => {
                content.push("Language keyword".to_string());
                content.push("Usage: Used for language syntax".to_string());
            }
            "Type/Class" => {
                content.push("Definition: (type definition would be shown here)".to_string());
                content.push("Members: (available methods/properties would be listed)".to_string());
            }
            _ => {
                content.push("Code identifier".to_string());
            }
        }

        // Add example usage
        content.push(String::new());
        content.push("Example:".to_string());
        content.push(format!("  {}", item.value));

        content
    }

    /// Check if preview has content
    pub fn has_content(&self) -> bool {
        !self.preview_content.is_empty()
    }

    /// Get current preview title
    pub fn title(&self) -> &str {
        &self.preview_title
    }
}

impl Default for CompletionPreview {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for CompletionPreview {
    async fn handle_key_event(&mut self, _event: KeyEvent) -> Result<()> {
        // Preview is read-only, no key handling needed
        Ok(())
    }

    async fn handle_mouse_event(&mut self, _event: MouseEvent) -> Result<()> {
        // Preview is read-only, no mouse handling needed
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.has_content() {
            return;
        }

        let preview_widget = Paragraph::new(self.preview_content.clone())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.preview_title.as_str())
                    .border_style(Style::default().fg(theme.colors.border))
                    .title_style(Style::default().fg(theme.colors.fg_base).add_modifier(Modifier::BOLD)),
            )
            .style(Style::default().fg(theme.colors.fg_base))
            .wrap(Wrap { trim: false });

        frame.render_widget(preview_widget, area);
    }

    fn size(&self) -> Rect {
        self.state.size
    }

    fn set_size(&mut self, size: Rect) {
        self.state.size = size;
    }

    fn has_focus(&self) -> bool {
        false // Preview never has focus
    }

    fn is_visible(&self) -> bool {
        self.state.is_visible && self.has_content()
    }

    fn set_visible(&mut self, visible: bool) {
        self.state.is_visible = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_preview_creation() {
        let preview = CompletionPreview::new();
        assert!(!preview.has_content());
        assert_eq!(preview.title(), "");
    }

    #[tokio::test]
    async fn test_preview_update() {
        let mut preview = CompletionPreview::new();
        
        let item = CompletionItem::new("test.rs", "test.rs", "file")
            .with_description("Rust source file".to_string());
        
        preview.update_preview(Some(item)).await.unwrap();
        
        assert!(preview.has_content());
        assert!(preview.title().contains("test.rs"));
        assert!(preview.preview_content.contains("test.rs"));
    }

    #[tokio::test]
    async fn test_file_preview() {
        let mut preview = CompletionPreview::new();
        
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "fn main() {{").unwrap();
        writeln!(temp_file, "    println!(\"Hello, world!\");").unwrap();
        writeln!(temp_file, "}}").unwrap();
        
        let file_path = temp_file.path().to_string_lossy().to_string();
        let content = preview.generate_file_preview(&file_path).await;
        
        assert!(!content.is_empty());
        assert!(content.iter().any(|line| line.contains("Type: File")));
    }

    #[tokio::test]
    async fn test_command_preview() {
        let preview = CompletionPreview::new();
        let content = preview.generate_command_preview("git").await;
        
        assert!(!content.is_empty());
        assert!(content.iter().any(|line| line.contains("Version control system")));
        assert!(content.iter().any(|line| line.contains("Usage Examples")));
    }

    #[tokio::test]
    async fn test_code_preview() {
        let preview = CompletionPreview::new();
        
        let item = CompletionItem::new("println!()", "println!", "code")
            .with_description("Print to stdout".to_string());
        
        let content = preview.generate_code_preview(&item).await;
        
        assert!(!content.is_empty());
        assert!(content.iter().any(|line| line.contains("Function/Method")));
    }

    #[test]
    fn test_preview_configuration() {
        let preview = CompletionPreview::new()
            .with_content_preview(false)
            .with_max_preview_lines(10)
            .with_metadata(false);
        
        assert!(!preview.show_content_preview);
        assert_eq!(preview.max_preview_lines, 10);
        assert!(!preview.show_metadata);
    }
}