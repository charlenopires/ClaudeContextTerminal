//! Chroma-style syntax highlighting integration
//! 
//! This module provides compatibility with Chroma-style highlighting
//! and integrates with the Goofy theme system, similar to how Crush
//! integrates with Chroma for syntax highlighting.

use super::{SyntaxHighlighter, HighlightedContent, HighlightConfig};
use crate::tui::themes::{Theme, current_theme};
use anyhow::Result;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use std::collections::HashMap;

/// Chroma-style syntax highlighter that integrates with Goofy themes
#[derive(Debug)]
pub struct ChromaHighlighter {
    /// Underlying syntax highlighter
    highlighter: SyntaxHighlighter,
    
    /// Theme-specific color mappings
    theme_mappings: HashMap<String, ChromaThemeMapping>,
    
    /// Current theme integration
    current_mapping: Option<ChromaThemeMapping>,
}

/// Mapping from syntax elements to theme colors
#[derive(Debug, Clone)]
struct ChromaThemeMapping {
    /// Background color for code blocks
    background: Color,
    
    /// Default text color
    text: Color,
    
    /// Comment color
    comment: Color,
    
    /// Keyword color (if, for, while, etc.)
    keyword: Color,
    
    /// String literal color
    string: Color,
    
    /// Number literal color
    number: Color,
    
    /// Function name color
    function: Color,
    
    /// Type name color
    type_name: Color,
    
    /// Operator color (+, -, *, etc.)
    operator: Color,
    
    /// Variable name color
    variable: Color,
    
    /// Constant color
    constant: Color,
    
    /// Error highlighting color
    error: Color,
    
    /// Line number color
    line_number: Color,
}

impl ChromaHighlighter {
    /// Create a new Chroma-style highlighter
    pub fn new() -> Result<Self> {
        let highlighter = SyntaxHighlighter::new()?;
        let mut chroma = Self {
            highlighter,
            theme_mappings: HashMap::new(),
            current_mapping: None,
        };
        
        chroma.initialize_theme_mappings();
        chroma.update_current_mapping();
        
        Ok(chroma)
    }
    
    /// Create with custom configuration
    pub fn with_config(config: HighlightConfig) -> Result<Self> {
        let highlighter = SyntaxHighlighter::with_config(config)?;
        let mut chroma = Self {
            highlighter,
            theme_mappings: HashMap::new(),
            current_mapping: None,
        };
        
        chroma.initialize_theme_mappings();
        chroma.update_current_mapping();
        
        Ok(chroma)
    }
    
    /// Highlight code using theme-integrated colors
    pub fn highlight(&mut self, code: &str, filename: Option<&str>) -> Result<HighlightedContent> {
        // Update theme mapping if theme has changed
        self.update_current_mapping();
        
        // Get basic highlighting
        let mut content = self.highlighter.highlight(code, filename)?;
        
        // Apply theme-specific styling if available
        if let Some(mapping) = &self.current_mapping {
            content = self.apply_theme_mapping(content, mapping);
        }
        
        Ok(content)
    }
    
    /// Highlight with explicit language
    pub fn highlight_language(&mut self, code: &str, language: &str) -> Result<HighlightedContent> {
        self.update_current_mapping();
        
        let mut content = self.highlighter.highlight_language(code, language)?;
        
        if let Some(mapping) = &self.current_mapping {
            content = self.apply_theme_mapping(content, mapping);
        }
        
        Ok(content)
    }
    
    /// Initialize theme mappings for different Goofy themes
    fn initialize_theme_mappings(&mut self) {
        // Goofy Dark theme mapping
        self.theme_mappings.insert("goofy_dark".to_string(), ChromaThemeMapping {
            background: Color::Rgb(0x2D, 0x2D, 0x2D),
            text: Color::Rgb(0xD0, 0xD0, 0xD0),
            comment: Color::Rgb(0x90, 0x90, 0x90),
            keyword: Color::Rgb(0x8A, 0x67, 0xFF),      // Primary purple
            string: Color::Rgb(0x9A, 0xE4, 0x78),       // Tertiary green
            number: Color::Rgb(0xFF, 0xE1, 0x9C),       // Secondary yellow
            function: Color::Rgb(0x29, 0xB6, 0xF6),     // Info blue
            type_name: Color::Rgb(0xFF, 0xA5, 0x00),    // Accent orange
            operator: Color::Rgb(0xD0, 0xD0, 0xD0),     // Base text
            variable: Color::Rgb(0xB0, 0xB0, 0xB0),     // Half-muted
            constant: Color::Rgb(0xFF, 0xE1, 0x9C),     // Secondary yellow
            error: Color::Rgb(0xF4, 0x43, 0x36),        // Error red
            line_number: Color::Rgb(0x90, 0x90, 0x90),  // Subtle
        });
        
        // Goofy Light theme mapping
        self.theme_mappings.insert("goofy_light".to_string(), ChromaThemeMapping {
            background: Color::Rgb(0xFD, 0xFD, 0xFD),
            text: Color::Rgb(0x20, 0x20, 0x20),
            comment: Color::Rgb(0x80, 0x86, 0x8B),
            keyword: Color::Rgb(0x67, 0x3A, 0xB7),      // Primary purple (darker)
            string: Color::Rgb(0x38, 0x8E, 0x3C),       // Tertiary green (darker)
            number: Color::Rgb(0xF5, 0x7C, 0x00),       // Secondary orange (darker)
            function: Color::Rgb(0x01, 0x65, 0xD4),     // Info blue (darker)
            type_name: Color::Rgb(0xD3, 0x2F, 0x2F),    // Accent red (darker)
            operator: Color::Rgb(0x20, 0x20, 0x20),     // Base text
            variable: Color::Rgb(0x40, 0x40, 0x40),     // Half-muted
            constant: Color::Rgb(0xF5, 0x7C, 0x00),     // Secondary orange
            error: Color::Rgb(0xC6, 0x28, 0x28),        // Error red (darker)
            line_number: Color::Rgb(0x80, 0x86, 0x8B),  // Subtle
        });
        
        // Classic Dark theme mapping
        self.theme_mappings.insert("classic_dark".to_string(), ChromaThemeMapping {
            background: Color::Black,
            text: Color::White,
            comment: Color::DarkGray,
            keyword: Color::Cyan,
            string: Color::Green,
            number: Color::Yellow,
            function: Color::Blue,
            type_name: Color::Magenta,
            operator: Color::White,
            variable: Color::White,
            constant: Color::Yellow,
            error: Color::Red,
            line_number: Color::DarkGray,
        });
        
        // Classic Light theme mapping
        self.theme_mappings.insert("classic_light".to_string(), ChromaThemeMapping {
            background: Color::White,
            text: Color::Black,
            comment: Color::DarkGray,
            keyword: Color::Blue,
            string: Color::Rgb(0x00, 0x80, 0x00),       // Dark green
            number: Color::Rgb(0xB8, 0x86, 0x00),       // Dark yellow
            function: Color::Blue,
            type_name: Color::Rgb(0x80, 0x00, 0x80),    // Dark magenta
            operator: Color::Black,
            variable: Color::Black,
            constant: Color::Rgb(0xB8, 0x86, 0x00),     // Dark yellow
            error: Color::Rgb(0x80, 0x00, 0x00),        // Dark red
            line_number: Color::Gray,
        });
    }
    
    /// Update current mapping based on active theme
    fn update_current_mapping(&mut self) {
        let theme = current_theme();
        self.current_mapping = self.theme_mappings.get(&theme.name).cloned();
    }
    
    /// Apply theme-specific color mapping to highlighted content
    fn apply_theme_mapping(
        &self, 
        mut content: HighlightedContent, 
        mapping: &ChromaThemeMapping
    ) -> HighlightedContent {
        // Apply theme colors to each line
        for line in &mut content.lines {
            for span in &mut line.spans {
                // Apply background color if it's a code span
                if !span.content.trim().is_empty() {
                    span.style = span.style.bg(mapping.background);
                }
                
                // Map colors based on content analysis
                // This is a simplified approach - in practice, you'd want
                // more sophisticated token type detection
                if self.looks_like_comment(&span.content) {
                    span.style = span.style.fg(mapping.comment);
                } else if self.looks_like_string(&span.content) {
                    span.style = span.style.fg(mapping.string);
                } else if self.looks_like_number(&span.content) {
                    span.style = span.style.fg(mapping.number);
                } else if self.looks_like_keyword(&span.content) {
                    span.style = span.style.fg(mapping.keyword);
                }
            }
        }
        
        content
    }
    
    /// Simple heuristic to detect comments
    fn looks_like_comment(&self, text: &str) -> bool {
        let trimmed = text.trim();
        trimmed.starts_with("//") || 
        trimmed.starts_with("/*") || 
        trimmed.starts_with('#') ||
        trimmed.starts_with("--")
    }
    
    /// Simple heuristic to detect string literals
    fn looks_like_string(&self, text: &str) -> bool {
        let trimmed = text.trim();
        (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
        (trimmed.starts_with('\'') && trimmed.ends_with('\'')) ||
        (trimmed.starts_with('`') && trimmed.ends_with('`'))
    }
    
    /// Simple heuristic to detect numbers
    fn looks_like_number(&self, text: &str) -> bool {
        text.trim().chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_')
            && text.trim().chars().any(|c| c.is_ascii_digit())
    }
    
    /// Simple heuristic to detect keywords
    fn looks_like_keyword(&self, text: &str) -> bool {
        matches!(text.trim(), 
            "fn" | "let" | "mut" | "const" | "static" | "if" | "else" | "for" | "while" | "loop" |
            "match" | "return" | "break" | "continue" | "struct" | "enum" | "trait" | "impl" |
            "pub" | "use" | "mod" | "crate" | "super" | "self" | "Self" | "async" | "await" |
            "function" | "var" | "const" | "class" | "def" | "import" | "from" | "as" |
            "public" | "private" | "protected" | "static" | "void" | "int" | "string" | "bool"
        )
    }
    
    /// Get current theme mapping
    pub fn current_mapping(&self) -> Option<&ChromaThemeMapping> {
        self.current_mapping.as_ref()
    }
    
    /// Force refresh of theme mapping
    pub fn refresh_theme(&mut self) {
        self.update_current_mapping();
    }
    
    /// Get supported languages
    pub fn supported_languages(&self) -> Vec<super::LanguageInfo> {
        self.highlighter.supported_languages()
    }
    
    /// Get available themes
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_mappings.keys().cloned().collect()
    }
    
    /// Set highlighting configuration
    pub fn set_config(&mut self, config: HighlightConfig) {
        self.highlighter.set_config(config);
    }
    
    /// Get current configuration
    pub fn config(&self) -> &HighlightConfig {
        self.highlighter.config()
    }
}

/// Helper function to create a Chroma-style highlighter for use in TUI components
pub fn create_theme_highlighter() -> Result<ChromaHighlighter> {
    ChromaHighlighter::new()
}

/// Highlight code block with current theme integration
pub fn highlight_code_block(code: &str, language: Option<&str>) -> Result<HighlightedContent> {
    let mut highlighter = ChromaHighlighter::new()?;
    
    match language {
        Some(lang) => highlighter.highlight_language(code, lang),
        None => highlighter.highlight(code, None),
    }
}

/// Highlight file content with filename-based detection
pub fn highlight_file_content(code: &str, filename: &str) -> Result<HighlightedContent> {
    let mut highlighter = ChromaHighlighter::new()?;
    highlighter.highlight(code, Some(filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chroma_highlighter_creation() {
        let highlighter = ChromaHighlighter::new();
        assert!(highlighter.is_ok());
    }
    
    #[test]
    fn test_theme_integration() {
        let mut highlighter = ChromaHighlighter::new().unwrap();
        
        let rust_code = r#"
fn main() {
    let message = "Hello, world!";
    println!("{}", message);
}
"#;
        
        let result = highlighter.highlight(rust_code, Some("main.rs"));
        assert!(result.is_ok());
        
        let highlighted = result.unwrap();
        assert!(!highlighted.lines.is_empty());
    }
    
    #[test] 
    fn test_theme_mappings() {
        let highlighter = ChromaHighlighter::new().unwrap();
        let themes = highlighter.available_themes();
        
        assert!(themes.contains(&"goofy_dark".to_string()));
        assert!(themes.contains(&"goofy_light".to_string()));
        assert!(themes.contains(&"classic_dark".to_string()));
        assert!(themes.contains(&"classic_light".to_string()));
    }
    
    #[test]
    fn test_content_detection() {
        let highlighter = ChromaHighlighter::new().unwrap();
        
        assert!(highlighter.looks_like_comment("// This is a comment"));
        assert!(highlighter.looks_like_comment("/* Block comment */"));
        assert!(highlighter.looks_like_comment("# Python comment"));
        
        assert!(highlighter.looks_like_string("\"hello world\""));
        assert!(highlighter.looks_like_string("'single quotes'"));
        assert!(highlighter.looks_like_string("`template string`"));
        
        assert!(highlighter.looks_like_number("123"));
        assert!(highlighter.looks_like_number("45.67"));
        assert!(highlighter.looks_like_number("1_000_000"));
        
        assert!(highlighter.looks_like_keyword("fn"));
        assert!(highlighter.looks_like_keyword("let"));
        assert!(highlighter.looks_like_keyword("function"));
    }
    
    #[test]
    fn test_convenience_functions() {
        let rust_code = "fn test() { println!(\"test\"); }";
        
        let result1 = highlight_code_block(rust_code, Some("rust"));
        assert!(result1.is_ok());
        
        let result2 = highlight_file_content(rust_code, "test.rs");
        assert!(result2.is_ok());
        
        let result3 = highlight_code_block(rust_code, None);
        assert!(result3.is_ok());
    }
}