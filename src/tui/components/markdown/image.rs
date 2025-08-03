//! Image support for markdown rendering
//! 
//! This module provides image handling capabilities for markdown,
//! including placeholder rendering and integration with the image widget.

use anyhow::Result;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use std::path::Path;

use super::styles::MarkdownStyles;
use crate::tui::components::image::{ImageWidget, ImageConfig};

/// Image placeholder configuration
#[derive(Debug, Clone)]
pub struct ImagePlaceholderConfig {
    /// Whether to show image dimensions
    pub show_dimensions: bool,
    
    /// Whether to show file size
    pub show_file_size: bool,
    
    /// Whether to show image format
    pub show_format: bool,
    
    /// Maximum placeholder width
    pub max_width: u16,
    
    /// Placeholder style
    pub style: ImagePlaceholderStyle,
}

/// Image placeholder styling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImagePlaceholderStyle {
    /// Simple text representation
    Simple,
    /// ASCII art box
    Box,
    /// Unicode symbols
    Unicode,
}

/// Image information extracted from URLs or paths
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// Image source (URL or file path)
    pub source: String,
    
    /// Alt text
    pub alt_text: String,
    
    /// Title text
    pub title: Option<String>,
    
    /// Detected file extension
    pub extension: Option<String>,
    
    /// Whether source appears to be a URL
    pub is_url: bool,
}

/// Image renderer for markdown
pub struct ImageRenderer {
    config: ImagePlaceholderConfig,
    styles: MarkdownStyles,
}

impl Default for ImagePlaceholderConfig {
    fn default() -> Self {
        Self {
            show_dimensions: true,
            show_file_size: false,
            show_format: true,
            max_width: 60,
            style: ImagePlaceholderStyle::Unicode,
        }
    }
}

impl ImageRenderer {
    /// Create a new image renderer
    pub fn new(config: ImagePlaceholderConfig, styles: MarkdownStyles) -> Self {
        Self { config, styles }
    }
    
    /// Render image placeholder to lines
    pub fn render_placeholder(&self, info: &ImageInfo) -> Result<Vec<Line<'static>>> {
        match self.config.style {
            ImagePlaceholderStyle::Simple => self.render_simple_placeholder(info),
            ImagePlaceholderStyle::Box => self.render_box_placeholder(info),
            ImagePlaceholderStyle::Unicode => self.render_unicode_placeholder(info),
        }
    }
    
    /// Render simple text placeholder
    fn render_simple_placeholder(&self, info: &ImageInfo) -> Result<Vec<Line<'static>>> {
        let mut lines = Vec::new();
        
        let prefix = if info.is_url { "🌐" } else { "🖼" };
        let main_text = if info.alt_text.is_empty() {
            format!("{} Image: {}", prefix, info.source)
        } else {
            format!("{} {}: {}", prefix, info.alt_text, info.source)
        };
        
        // Truncate if too long
        let truncated_text = if main_text.chars().count() > self.config.max_width as usize {
            let mut truncated = main_text.chars()
                .take(self.config.max_width as usize - 3)
                .collect::<String>();
            truncated.push_str("...");
            truncated
        } else {
            main_text
        };
        
        let span = Span::styled(truncated_text, self.styles.image);
        lines.push(Line::from(span));
        
        // Add additional info if requested
        if let Some(title) = &info.title {
            if !title.is_empty() {
                let title_span = Span::styled(
                    format!("  Title: {}", title),
                    self.styles.image.fg(Color::Gray)
                );
                lines.push(Line::from(title_span));
            }
        }
        
        if self.config.show_format {
            if let Some(extension) = &info.extension {
                let format_span = Span::styled(
                    format!("  Format: {}", extension.to_uppercase()),
                    self.styles.image.fg(Color::Gray)
                );
                lines.push(Line::from(format_span));
            }
        }
        
        Ok(lines)
    }
    
    /// Render box-style placeholder
    fn render_box_placeholder(&self, info: &ImageInfo) -> Result<Vec<Line<'static>>> {
        let mut lines = Vec::new();
        
        let box_width = (self.config.max_width as usize).min(60);
        let border_style = self.styles.image;
        
        // Top border
        let top_border = format!("┌{}┐", "─".repeat(box_width - 2));
        lines.push(Line::from(Span::styled(top_border, border_style)));
        
        // Content lines
        let prefix = if info.is_url { "🌐" } else { "🖼" };
        let content_lines = if info.alt_text.is_empty() {
            vec![
                format!("{} Image", prefix),
                info.source.clone(),
            ]
        } else {
            vec![
                format!("{} {}", prefix, info.alt_text),
                info.source.clone(),
            ]
        };
        
        for content_line in content_lines {
            let truncated = if content_line.chars().count() > box_width - 4 {
                let mut truncated = content_line.chars()
                    .take(box_width - 7)
                    .collect::<String>();
                truncated.push_str("...");
                truncated
            } else {
                content_line
            };
            
            let padding_needed = box_width - 4 - truncated.chars().count();
            let padded_content = format!(
                "│ {}{} │",
                truncated,
                " ".repeat(padding_needed)
            );
            
            lines.push(Line::from(Span::styled(padded_content, border_style)));
        }
        
        // Additional info
        if let Some(title) = &info.title {
            if !title.is_empty() {
                let title_line = format!("Title: {}", title);
                let truncated = if title_line.chars().count() > box_width - 4 {
                    let mut truncated = title_line.chars()
                        .take(box_width - 7)
                        .collect::<String>();
                    truncated.push_str("...");
                    truncated
                } else {
                    title_line
                };
                
                let padding_needed = box_width - 4 - truncated.chars().count();
                let padded_content = format!(
                    "│ {}{} │",
                    truncated,
                    " ".repeat(padding_needed)
                );
                
                lines.push(Line::from(Span::styled(padded_content, border_style.fg(Color::Gray))));
            }
        }
        
        // Bottom border
        let bottom_border = format!("└{}┘", "─".repeat(box_width - 2));
        lines.push(Line::from(Span::styled(bottom_border, border_style)));
        
        Ok(lines)
    }
    
    /// Render Unicode-style placeholder
    fn render_unicode_placeholder(&self, info: &ImageInfo) -> Result<Vec<Line<'static>>> {
        let mut lines = Vec::new();
        
        // Create a decorative frame using Unicode box drawing characters
        let frame_width = (self.config.max_width as usize).min(50);
        
        // Top decoration
        let top_decoration = format!("╭{}╮", "─".repeat(frame_width - 2));
        lines.push(Line::from(Span::styled(top_decoration, self.styles.image)));
        
        // Main content with icon
        let icon = if info.is_url {
            "🌐"
        } else {
            match info.extension.as_deref() {
                Some("png") | Some("PNG") => "🖼",
                Some("jpg") | Some("jpeg") | Some("JPG") | Some("JPEG") => "📷",
                Some("gif") | Some("GIF") => "🎞",
                Some("svg") | Some("SVG") => "🎨",
                Some("webp") | Some("WEBP") => "🖼",
                _ => "🖼",
            }
        };
        
        let main_content = if info.alt_text.is_empty() {
            format!("{} Image", icon)
        } else {
            format!("{} {}", icon, info.alt_text)
        };
        
        let content_padding = frame_width.saturating_sub(main_content.chars().count() + 4);
        let content_line = format!(
            "│ {}{} │",
            main_content,
            " ".repeat(content_padding)
        );
        lines.push(Line::from(Span::styled(content_line, self.styles.image)));
        
        // Source line
        let source_display = if info.source.chars().count() > frame_width - 6 {
            let mut truncated = info.source.chars()
                .take(frame_width - 9)
                .collect::<String>();
            truncated.push_str("...");
            truncated
        } else {
            info.source.clone()
        };
        
        let source_padding = frame_width.saturating_sub(source_display.chars().count() + 4);
        let source_line = format!(
            "│ {}{} │",
            source_display,
            " ".repeat(source_padding)
        );
        lines.push(Line::from(Span::styled(source_line, self.styles.image.fg(Color::Gray))));
        
        // Format info if enabled
        if self.config.show_format {
            if let Some(extension) = &info.extension {
                let format_text = format!("Format: {}", extension.to_uppercase());
                let format_padding = frame_width.saturating_sub(format_text.chars().count() + 4);
                let format_line = format!(
                    "│ {}{} │",
                    format_text,
                    " ".repeat(format_padding)
                );
                lines.push(Line::from(Span::styled(format_line, self.styles.image.fg(Color::DarkGray))));
            }
        }
        
        // Bottom decoration
        let bottom_decoration = format!("╰{}╯", "─".repeat(frame_width - 2));
        lines.push(Line::from(Span::styled(bottom_decoration, self.styles.image)));
        
        Ok(lines)
    }
}

/// Utility functions for image processing
pub mod utils {
    use super::*;
    
    /// Parse image information from markdown image syntax
    pub fn parse_image_info(alt_text: &str, source: &str, title: Option<&str>) -> ImageInfo {
        let is_url = source.starts_with("http://") || source.starts_with("https://") || source.starts_with("ftp://");
        
        let extension = if is_url {
            // Try to extract extension from URL
            let url_path = source.split('?').next().unwrap_or(source);
            Path::new(url_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase())
        } else {
            // Extract from file path
            Path::new(source)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase())
        };
        
        ImageInfo {
            source: source.to_string(),
            alt_text: alt_text.to_string(),
            title: title.map(|t| t.to_string()),
            extension,
            is_url,
        }
    }
    
    /// Check if a file extension indicates an image
    pub fn is_image_extension(extension: &str) -> bool {
        match extension.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "tif" |
            "webp" | "svg" | "ico" | "avif" | "heic" | "heif" => true,
            _ => false,
        }
    }
    
    /// Get appropriate icon for image type
    pub fn get_image_icon(extension: Option<&str>) -> &'static str {
        match extension {
            Some("png") | Some("PNG") => "🖼",
            Some("jpg") | Some("jpeg") | Some("JPG") | Some("JPEG") => "📷",
            Some("gif") | Some("GIF") => "🎞",
            Some("svg") | Some("SVG") => "🎨",
            Some("bmp") | Some("BMP") => "🖼",
            Some("tiff") | Some("tif") | Some("TIFF") | Some("TIF") => "📷",
            Some("webp") | Some("WEBP") => "🖼",
            Some("ico") | Some("ICO") => "🎯",
            Some("avif") | Some("AVIF") => "🖼",
            Some("heic") | Some("heif") | Some("HEIC") | Some("HEIF") => "📱",
            _ => "🖼",
        }
    }
    
    /// Extract dimensions from image alt text or title
    pub fn extract_dimensions(text: &str) -> Option<(u32, u32)> {
        // Look for patterns like "1920x1080", "800×600", "640 x 480"
        let re = regex::Regex::new(r"(\d+)\s*[×x]\s*(\d+)").ok()?;
        
        if let Some(captures) = re.captures(text) {
            let width = captures.get(1)?.as_str().parse().ok()?;
            let height = captures.get(2)?.as_str().parse().ok()?;
            Some((width, height))
        } else {
            None
        }
    }
    
    /// Create a fallback text representation for images
    pub fn create_fallback_text(info: &ImageInfo) -> String {
        if info.alt_text.is_empty() {
            format!("[Image: {}]", info.source)
        } else {
            format!("[{}: {}]", info.alt_text, info.source)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_image_info_parsing() {
        let info = utils::parse_image_info(
            "A beautiful sunset",
            "https://example.com/sunset.jpg",
            Some("Sunset over the ocean")
        );
        
        assert_eq!(info.alt_text, "A beautiful sunset");
        assert!(info.is_url);
        assert_eq!(info.extension, Some("jpg".to_string()));
        assert_eq!(info.title, Some("Sunset over the ocean".to_string()));
    }
    
    #[test]
    fn test_local_file_parsing() {
        let info = utils::parse_image_info(
            "Local image",
            "/path/to/image.png",
            None
        );
        
        assert!(!info.is_url);
        assert_eq!(info.extension, Some("png".to_string()));
        assert_eq!(info.title, None);
    }
    
    #[test]
    fn test_image_extension_detection() {
        assert!(utils::is_image_extension("jpg"));
        assert!(utils::is_image_extension("PNG"));
        assert!(utils::is_image_extension("gif"));
        assert!(!utils::is_image_extension("txt"));
        assert!(!utils::is_image_extension("pdf"));
    }
    
    #[test]
    fn test_icon_selection() {
        assert_eq!(utils::get_image_icon(Some("jpg")), "📷");
        assert_eq!(utils::get_image_icon(Some("png")), "🖼");
        assert_eq!(utils::get_image_icon(Some("gif")), "🎞");
        assert_eq!(utils::get_image_icon(Some("svg")), "🎨");
        assert_eq!(utils::get_image_icon(None), "🖼");
    }
    
    #[test]
    fn test_dimension_extraction() {
        assert_eq!(utils::extract_dimensions("Image 1920x1080"), Some((1920, 1080)));
        assert_eq!(utils::extract_dimensions("Size: 800×600 pixels"), Some((800, 600)));
        assert_eq!(utils::extract_dimensions("640 x 480"), Some((640, 480)));
        assert_eq!(utils::extract_dimensions("No dimensions here"), None);
    }
    
    #[test]
    fn test_fallback_text() {
        let info = ImageInfo {
            source: "test.jpg".to_string(),
            alt_text: "Test image".to_string(),
            title: None,
            extension: Some("jpg".to_string()),
            is_url: false,
        };
        
        let fallback = utils::create_fallback_text(&info);
        assert_eq!(fallback, "[Test image: test.jpg]");
        
        let info_no_alt = ImageInfo {
            source: "test.jpg".to_string(),
            alt_text: "".to_string(),
            title: None,
            extension: Some("jpg".to_string()),
            is_url: false,
        };
        
        let fallback_no_alt = utils::create_fallback_text(&info_no_alt);
        assert_eq!(fallback_no_alt, "[Image: test.jpg]");
    }
    
    #[test]
    fn test_placeholder_config() {
        let config = ImagePlaceholderConfig::default();
        assert!(config.show_dimensions);
        assert!(config.show_format);
        assert_eq!(config.style, ImagePlaceholderStyle::Unicode);
    }
}