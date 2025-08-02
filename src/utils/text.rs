// Text processing utilities

use anyhow::{Context, Result};
use pulldown_cmark::{Parser, html, Options};
use syntect::{
    parsing::SyntaxSet,
    highlighting::{ThemeSet, Style},
    util::as_24_bit_terminal_escaped,
    easy::HighlightLines,
};
use std::collections::HashMap;
use comrak::{markdown_to_html, ComrakOptions};

/// Markdown processing utilities
pub mod markdown {
    use super::*;

    /// Convert markdown to HTML
    pub fn to_html(markdown: &str) -> String {
        let mut options = ComrakOptions::default();
        options.extension.strikethrough = true;
        options.extension.tagfilter = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.superscript = true;
        options.extension.header_ids = Some("".to_string());
        options.extension.footnotes = true;
        
        markdown_to_html(markdown, &options)
    }
    
    /// Convert HTML back to markdown (simple conversion)
    pub fn from_html(html: &str) -> Result<String> {
        Ok(html2md::parse_html(html))
    }
    
    /// Extract plain text from markdown
    pub fn to_plain_text(markdown: &str) -> String {
        let parser = Parser::new(markdown);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        // Remove HTML tags for plain text
        html2md::parse_html(&html_output)
    }
    
    /// Count words in markdown text
    pub fn count_words(markdown: &str) -> usize {
        let plain_text = to_plain_text(markdown);
        plain_text
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .count()
    }
    
    /// Extract headers from markdown
    pub fn extract_headers(markdown: &str) -> Vec<(u32, String)> {
        let mut headers = Vec::new();
        
        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count() as u32;
                if level <= 6 {
                    let title = trimmed.trim_start_matches('#').trim().to_string();
                    if !title.is_empty() {
                        headers.push((level, title));
                    }
                }
            }
        }
        
        headers
    }
    
    /// Create a table of contents from markdown
    pub fn create_toc(markdown: &str) -> String {
        let headers = extract_headers(markdown);
        let mut toc = String::new();
        
        for (level, title) in headers {
            let indent = "  ".repeat((level.saturating_sub(1)) as usize);
            let anchor = title
                .to_lowercase()
                .replace(' ', "-")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>();
            
            toc.push_str(&format!("{}* [{}](#{})\n", indent, title, anchor));
        }
        
        toc
    }
}

/// Syntax highlighting utilities
pub mod syntax {
    use super::*;
    
    /// Language syntax highlighter
    pub struct SyntaxHighlighter {
        syntax_set: SyntaxSet,
        theme_set: ThemeSet,
    }
    
    impl Default for SyntaxHighlighter {
        fn default() -> Self {
            Self {
                syntax_set: SyntaxSet::load_defaults_newlines(),
                theme_set: ThemeSet::load_defaults(),
            }
        }
    }
    
    impl SyntaxHighlighter {
        /// Create a new syntax highlighter
        pub fn new() -> Self {
            Self::default()
        }
        
        /// Get available language names
        pub fn get_languages(&self) -> Vec<&str> {
            self.syntax_set
                .syntaxes()
                .iter()
                .map(|syntax| syntax.name.as_str())
                .collect()
        }
        
        /// Get available theme names
        pub fn get_themes(&self) -> Vec<&str> {
            self.theme_set.themes.keys().map(|s| s.as_str()).collect()
        }
        
        /// Highlight code with the specified language and theme
        pub fn highlight(
            &self,
            code: &str,
            language: Option<&str>,
            theme_name: Option<&str>,
        ) -> Result<String> {
            let syntax = if let Some(lang) = language {
                self.syntax_set
                    .find_syntax_by_name(lang)
                    .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
                    .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
            } else {
                self.syntax_set.find_syntax_plain_text()
            };
            
            let theme = &self.theme_set.themes[theme_name.unwrap_or("base16-ocean.dark")];
            let mut highlighter = HighlightLines::new(syntax, theme);
            
            let mut result = String::new();
            for line in code.lines() {
                let ranges = highlighter
                    .highlight_line(line, &self.syntax_set)
                    .with_context(|| "Failed to highlight line")?;
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                result.push_str(&escaped);
                result.push('\n');
            }
            
            Ok(result)
        }
        
        /// Detect language from file extension
        pub fn detect_language(&self, file_extension: &str) -> Option<&str> {
            self.syntax_set
                .find_syntax_by_extension(file_extension)
                .map(|syntax| syntax.name.as_str())
        }
        
        /// Highlight code block and return terminal-friendly output
        pub fn highlight_for_terminal(
            &self,
            code: &str,
            language: Option<&str>,
        ) -> String {
            self.highlight(code, language, Some("base16-ocean.dark"))
                .unwrap_or_else(|_| code.to_string())
        }
    }
}

/// String and text manipulation utilities
pub mod string {
    use super::*;
    
    /// Truncate text to a specified length with ellipsis
    pub fn truncate(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else if max_length <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &text[..max_length - 3])
        }
    }
    
    /// Word wrap text to specified width
    pub fn word_wrap(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }
        
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_length = 0;
        
        for word in text.split_whitespace() {
            let word_len = word.len();
            
            if current_length == 0 {
                // First word on line
                current_line.push_str(word);
                current_length = word_len;
            } else if current_length + 1 + word_len <= width {
                // Word fits on current line
                current_line.push(' ');
                current_line.push_str(word);
                current_length += 1 + word_len;
            } else {
                // Word doesn't fit, start new line
                lines.push(current_line);
                current_line = word.to_string();
                current_length = word_len;
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
    
    /// Escape special characters for shell safety
    pub fn shell_escape(text: &str) -> String {
        if text.chars().all(|c| c.is_alphanumeric() || "-_./".contains(c)) {
            text.to_string()
        } else {
            format!("'{}'", text.replace('\'', "'\"'\"'"))
        }
    }
    
    /// Clean and normalize whitespace
    pub fn normalize_whitespace(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    /// Remove ANSI escape codes from text
    pub fn strip_ansi_codes(text: &str) -> String {
        let ansi_regex = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        ansi_regex.replace_all(text, "").to_string()
    }
    
    /// Convert string to title case
    pub fn to_title_case(text: &str) -> String {
        text.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Extract URLs from text
    pub fn extract_urls(text: &str) -> Vec<String> {
        let url_regex = regex::Regex::new(
            r"https?://(?:[-\w.])+(?::[0-9]+)?(?:/(?:[\w/_.])*(?:\?(?:[\w&=%.])*)?(?:#(?:[\w.])*)?)?",
        ).unwrap();
        
        url_regex
            .find_iter(text)
            .map(|mat| mat.as_str().to_string())
            .collect()
    }
    
    /// Calculate text similarity using Levenshtein distance
    pub fn similarity(text1: &str, text2: &str) -> f64 {
        let len1 = text1.len();
        let len2 = text2.len();
        
        if len1 == 0 && len2 == 0 {
            return 1.0;
        }
        
        let max_len = len1.max(len2);
        let distance = levenshtein_distance(text1, text2);
        
        1.0 - (distance as f64 / max_len as f64)
    }
    
    /// Find common prefix of two strings
    pub fn common_prefix(text1: &str, text2: &str) -> String {
        let chars1: Vec<char> = text1.chars().collect();
        let chars2: Vec<char> = text2.chars().collect();
        
        let mut prefix = String::new();
        let min_len = chars1.len().min(chars2.len());
        
        for i in 0..min_len {
            if chars1[i] == chars2[i] {
                prefix.push(chars1[i]);
            } else {
                break;
            }
        }
        
        prefix
    }
    
    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }
        
        matrix[len1][len2]
    }
}

/// Text formatting utilities
pub mod format {
    use super::*;
    
    /// Format file size in human-readable format
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = size as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
    
    /// Format duration in human-readable format
    pub fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        let millis = duration.subsec_millis();
        
        if secs >= 3600 {
            format!("{}h {}m {}s", secs / 3600, (secs % 3600) / 60, secs % 60)
        } else if secs >= 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else if secs > 0 {
            format!("{}s", secs)
        } else {
            format!("{}ms", millis)
        }
    }
    
    /// Create a progress bar string
    pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
        if total == 0 {
            return "█".repeat(width);
        }
        
        let progress = (current as f64 / total as f64).min(1.0);
        let filled = (progress * width as f64) as usize;
        let empty = width.saturating_sub(filled);
        
        format!("{}{}", "█".repeat(filled), "░".repeat(empty))
    }
    
    /// Format a table with aligned columns
    pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
        if headers.is_empty() || rows.is_empty() {
            return String::new();
        }
        
        // Calculate column widths
        let mut widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();
        
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }
        
        let mut result = String::new();
        
        // Header row
        let header_row = headers
            .iter()
            .enumerate()
            .map(|(i, header)| format!("{:<width$}", header, width = widths[i]))
            .collect::<Vec<_>>()
            .join(" | ");
        result.push_str(&header_row);
        result.push('\n');
        
        // Separator row
        let separator = widths
            .iter()
            .map(|&width| "-".repeat(width))
            .collect::<Vec<_>>()
            .join("-|-");
        result.push_str(&separator);
        result.push('\n');
        
        // Data rows
        for row in rows {
            let formatted_row = row
                .iter()
                .enumerate()
                .map(|(i, cell)| {
                    let width = widths.get(i).copied().unwrap_or(0);
                    format!("{:<width$}", cell, width = width)
                })
                .collect::<Vec<_>>()
                .join(" | ");
            result.push_str(&formatted_row);
            result.push('\n');
        }
        
        result
    }
}

/// Template processing utilities
pub mod template {
    use super::*;
    
    /// Simple template engine for variable substitution
    pub struct SimpleTemplate {
        variables: HashMap<String, String>,
    }
    
    impl SimpleTemplate {
        pub fn new() -> Self {
            Self {
                variables: HashMap::new(),
            }
        }
        
        /// Set a template variable
        pub fn set(&mut self, key: &str, value: &str) {
            self.variables.insert(key.to_string(), value.to_string());
        }
        
        /// Set multiple variables from a HashMap
        pub fn set_all(&mut self, variables: HashMap<String, String>) {
            self.variables.extend(variables);
        }
        
        /// Render template with variable substitution
        /// Variables are specified as {{variable_name}}
        pub fn render(&self, template: &str) -> String {
            let mut result = template.to_string();
            
            for (key, value) in &self.variables {
                let placeholder = format!("{{{{{}}}}}", key);
                result = result.replace(&placeholder, value);
            }
            
            result
        }
        
        /// Extract variables from template
        pub fn extract_variables(&self, template: &str) -> Vec<String> {
            let regex = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
            regex
                .captures_iter(template)
                .map(|cap| cap[1].trim().to_string())
                .collect()
        }
    }
    
    impl Default for SimpleTemplate {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_html() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = markdown::to_html(markdown);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn test_word_wrap() {
        let text = "The quick brown fox jumps over the lazy dog";
        let wrapped = string::word_wrap(text, 20);
        assert!(wrapped.len() > 1);
        assert!(wrapped.iter().all(|line| line.len() <= 20));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(string::truncate("Hello World", 8), "Hello...");
        assert_eq!(string::truncate("Hi", 10), "Hi");
    }

    #[test]
    fn test_template_rendering() {
        let mut template = template::SimpleTemplate::new();
        template.set("name", "World");
        template.set("greeting", "Hello");
        
        let result = template.render("{{greeting}}, {{name}}!");
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(format::format_file_size(1024), "1.0 KB");
        assert_eq!(format::format_file_size(1536), "1.5 KB");
        assert_eq!(format::format_file_size(512), "512 B");
    }
}