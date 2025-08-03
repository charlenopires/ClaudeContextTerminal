//! Goofy logo component for TUI display
//! 
//! This module provides ASCII art rendering of the GOOFY logo with gradient support.
//! Inspired by Charmbracelet's Crush logo component but adapted for Goofy branding.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use crate::tui::themes::colors::{ColorPalette, manipulate};

/// Options for rendering the Goofy logo
#[derive(Debug, Clone)]
pub struct LogoOpts {
    /// Primary gradient color (start)
    pub gradient_start: Color,
    /// Secondary gradient color (end)  
    pub gradient_end: Color,
    /// Background diagonal pattern color
    pub field_color: Color,
    /// Brand text color (e.g., "Goofy™")
    pub brand_color: Color,
    /// Version text color
    pub version_color: Color,
    /// Maximum width for the logo
    pub width: usize,
    /// Whether to render in compact mode
    pub compact: bool,
}

impl Default for LogoOpts {
    fn default() -> Self {
        Self {
            gradient_start: ColorPalette::GOOFY_ORANGE,
            gradient_end: ColorPalette::GOOFY_PURPLE,
            field_color: ColorPalette::GOOFY_ORANGE,
            brand_color: ColorPalette::GOOFY_PURPLE,
            version_color: ColorPalette::GOOFY_ORANGE,
            width: 80,
            compact: false,
        }
    }
}

/// Represents a single letter in ASCII art form
type LetterForm = fn(stretch: bool) -> Vec<String>;

/// The diagonal character used for background patterns
const DIAG: &str = "╱";

/// Render the complete Goofy logo with version and branding
pub fn render_logo(version: &str, opts: LogoOpts) -> Text<'static> {
    let brand_text = " Goofy™";
    
    // Generate the main GOOFY ASCII art
    let letters: Vec<LetterForm> = vec![
        letter_g,
        letter_o,
        letter_o,
        letter_f,
        letter_y,
    ];
    
    let spacing = if opts.compact { 0 } else { 1 };
    let stretch_index = if opts.compact { None } else { Some(2) }; // Stretch second 'O'
    
    let logo_lines = render_word(&letters, spacing, stretch_index);
    let logo_width = logo_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    
    // Apply gradient to the logo
    let gradient_logo = apply_gradient_to_lines(&logo_lines, opts.gradient_start, opts.gradient_end);
    
    // Create meta row (brand + version)
    let version_truncated = if version.len() + brand_text.len() + 1 > logo_width {
        let max_version_len = logo_width.saturating_sub(brand_text.len() + 1);
        if version.len() > max_version_len {
            format!("{}…", &version[..max_version_len.saturating_sub(1)])
        } else {
            version.to_string()
        }
    } else {
        version.to_string()
    };
    
    let gap_size = logo_width.saturating_sub(brand_text.len() + version_truncated.len());
    let gap = " ".repeat(gap_size);
    
    let meta_line = Line::from(vec![
        Span::styled(brand_text, Style::default().fg(opts.brand_color)),
        Span::raw(gap),
        Span::styled(version_truncated, Style::default().fg(opts.version_color)),
    ]);
    
    // Build the complete logo
    let mut text_lines = vec![meta_line];
    text_lines.extend(gradient_logo);
    
    // Add background fields if not compact
    if !opts.compact {
        text_lines = add_background_fields(text_lines, logo_width, opts.field_color, opts.width);
    }
    
    Text::from(text_lines)
}

/// Render a smaller version of the logo for constrained spaces
pub fn render_small_logo(width: usize, opts: LogoOpts) -> Line<'static> {
    let brand = "Goofy™";
    let brand_span = Span::styled(brand, Style::default().fg(opts.brand_color));
    
    let remaining_width = width.saturating_sub(brand.len() + 1);
    let field_pattern = DIAG.repeat(remaining_width);
    let field_span = Span::styled(field_pattern, Style::default().fg(opts.field_color));
    
    Line::from(vec![brand_span, Span::raw(" "), field_span])
}

/// Apply gradient coloring to logo lines
fn apply_gradient_to_lines(lines: &[String], start: Color, end: Color) -> Vec<Line<'static>> {
    let total_chars: usize = lines.iter().map(|line| line.chars().count()).sum();
    let gradient_colors = manipulate::linear_gradient(start, end, total_chars);
    
    let mut color_index = 0;
    let mut result_lines = Vec::new();
    
    for line in lines {
        let mut spans = Vec::new();
        for ch in line.chars() {
            if ch == ' ' {
                spans.push(Span::raw(" "));
            } else {
                let color = gradient_colors.get(color_index).copied().unwrap_or(start);
                spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                color_index += 1;
            }
        }
        result_lines.push(Line::from(spans));
    }
    
    result_lines
}

/// Add diagonal background fields to the logo
fn add_background_fields(
    mut logo_lines: Vec<Line<'static>>, 
    logo_width: usize, 
    field_color: Color, 
    total_width: usize
) -> Vec<Line<'static>> {
    let field_height = logo_lines.len();
    let left_width = 6;
    let right_width = total_width.saturating_sub(logo_width + left_width + 2).max(15);
    
    // Create left and right field patterns
    for (i, line) in logo_lines.iter_mut().enumerate() {
        let left_field = Span::styled(
            DIAG.repeat(left_width), 
            Style::default().fg(field_color)
        );
        
        // Right field with step-down effect
        let right_field_width = right_width.saturating_sub(i);
        let right_field = Span::styled(
            DIAG.repeat(right_field_width),
            Style::default().fg(field_color)
        );
        
        // Rebuild the line with fields
        let mut new_spans = vec![left_field, Span::raw(" ")];
        new_spans.extend(line.spans.clone());
        new_spans.extend(vec![Span::raw(" "), right_field]);
        
        *line = Line::from(new_spans);
    }
    
    logo_lines
}

/// Render multiple letters into lines of text
fn render_word(letters: &[LetterForm], spacing: usize, stretch_index: Option<usize>) -> Vec<String> {
    let rendered_letters: Vec<Vec<String>> = letters
        .iter()
        .enumerate()
        .map(|(i, letter)| {
            let should_stretch = stretch_index.map_or(false, |idx| i == idx);
            letter(should_stretch)
        })
        .collect();
    
    // Find the maximum height
    let max_height = rendered_letters
        .iter()
        .map(|letter| letter.len())
        .max()
        .unwrap_or(0);
    
    // Combine letters horizontally
    let mut result = Vec::new();
    for row in 0..max_height {
        let mut line = String::new();
        
        for (i, letter) in rendered_letters.iter().enumerate() {
            // Add spacing between letters
            if i > 0 {
                line.push_str(&" ".repeat(spacing));
            }
            
            // Add the character from this letter at this row
            if row < letter.len() {
                line.push_str(&letter[row]);
            } else {
                // Pad with spaces if this letter is shorter
                let width = letter.get(0).map_or(0, |s| s.len());
                line.push_str(&" ".repeat(width));
            }
        }
        result.push(line);
    }
    
    result
}

/// ASCII art for letter G
fn letter_g(stretch: bool) -> Vec<String> {
    let base_width = if stretch { 7 } else { 5 };
    let top = format!("▄{}▄", "▀".repeat(base_width - 2));
    let middle = format!("█{}▄", " ".repeat(base_width - 2));
    let bottom = format!("▀{}▀", "▀".repeat(base_width - 2));
    
    vec![top, middle, bottom]
}

/// ASCII art for letter O
fn letter_o(stretch: bool) -> Vec<String> {
    let base_width = if stretch { 6 } else { 4 };
    let top = format!("▄{}▄", "▀".repeat(base_width - 2));
    let middle = format!("█{}█", " ".repeat(base_width - 2));
    let bottom = format!("▀{}▀", "▀".repeat(base_width - 2));
    
    vec![top, middle, bottom]
}

/// ASCII art for letter F
fn letter_f(_stretch: bool) -> Vec<String> {
    vec![
        "█▀▀▀".to_string(),
        "█▀▀ ".to_string(),
        "▀   ".to_string(),
    ]
}

/// ASCII art for letter Y
fn letter_y(_stretch: bool) -> Vec<String> {
    vec![
        "█ █".to_string(),
        " █ ".to_string(),
        " ▀ ".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_rendering() {
        let opts = LogoOpts::default();
        let logo = render_logo("v1.0.0", opts);
        assert!(!logo.lines.is_empty());
    }

    #[test]
    fn test_small_logo_rendering() {
        let opts = LogoOpts::default();
        let small_logo = render_small_logo(40, opts);
        assert!(!small_logo.spans.is_empty());
    }

    #[test]
    fn test_word_rendering() {
        let letters = vec![letter_g, letter_o];
        let result = render_word(&letters, 1, None);
        assert_eq!(result.len(), 3); // Should have 3 rows
    }
}