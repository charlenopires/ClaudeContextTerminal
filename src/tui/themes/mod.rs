//! Advanced theming system for Goofy TUI
//! 
//! This module provides a comprehensive theming system with support for
//! color schemes, styles, animations, and responsive design.
//! 
//! Based on the Charmbracelet Crush theme architecture, this module
//! provides a complete theming solution with semantic color definitions,
//! pre-built component styles, and theme management.

use std::collections::HashMap;
use ratatui::style::{Color, Style, Modifier};
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub mod colors;
pub mod styles;
pub mod presets;

/// Theme represents a complete visual style configuration
/// 
/// This structure closely mirrors the Crush theme implementation,
/// providing comprehensive color definitions and semantic naming.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    
    // Primary brand colors
    pub primary: Color,
    pub secondary: Color, 
    pub tertiary: Color,
    pub accent: Color,
    
    // Background colors with semantic naming
    pub bg_base: Color,
    pub bg_base_lighter: Color,
    pub bg_subtle: Color,
    pub bg_overlay: Color,
    
    // Foreground colors for text and UI elements
    pub fg_base: Color,
    pub fg_muted: Color,
    pub fg_half_muted: Color,
    pub fg_subtle: Color,
    pub fg_selected: Color,
    
    // Border colors
    pub border: Color,
    pub border_focus: Color,
    
    // Status and semantic colors
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    
    // Extended color palette
    pub white: Color,
    pub blue_light: Color,
    pub blue: Color,
    pub yellow: Color,
    pub green: Color,
    pub green_dark: Color,
    pub green_light: Color,
    pub red: Color,
    pub red_dark: Color,
    pub red_light: Color,
    pub cherry: Color,
    
    // Cached styles - built lazily
    styles: Option<Styles>,
}

/// Pre-built styles for UI components
/// 
/// This structure provides ready-to-use styles for various UI components,
/// similar to the Crush Styles struct. Styles are built from the theme colors
/// and cached for performance.
#[derive(Debug, Clone)]
pub struct Styles {
    // Base styles
    pub base: Style,
    pub selected_base: Style,
    
    // Typography styles
    pub title: Style,
    pub subtitle: Style,
    pub text: Style,
    pub text_selected: Style,
    pub muted: Style,
    pub subtle: Style,
    
    // Status styles
    pub success: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    
    // Input component styles
    pub text_input_focused: Style,
    pub text_input_blurred: Style,
    pub text_input_placeholder: Style,
    pub text_input_prompt: Style,
    pub text_input_cursor: Style,
    
    // Text area styles
    pub text_area_focused: Style,
    pub text_area_blurred: Style,
    pub text_area_line_number: Style,
    pub text_area_cursor_line: Style,
    
    // Help system styles
    pub help_short_key: Style,
    pub help_short_desc: Style,
    pub help_short_separator: Style,
    pub help_ellipsis: Style,
    pub help_full_key: Style,
    pub help_full_desc: Style,
    pub help_full_separator: Style,
    
    // Dialog styles
    pub dialog_border: Style,
    pub dialog_title: Style,
    pub dialog_content: Style,
    
    // List styles
    pub list_item: Style,
    pub list_item_selected: Style,
    pub list_item_focused: Style,
    
    // File picker styles
    pub file_picker_cursor: Style,
    pub file_picker_directory: Style,
    pub file_picker_file: Style,
    pub file_picker_symlink: Style,
    pub file_picker_selected: Style,
    pub file_picker_disabled: Style,
    pub file_picker_permission: Style,
    pub file_picker_size: Style,
    
    // Diff viewer styles
    pub diff_equal_line: Style,
    pub diff_insert_line: Style,
    pub diff_delete_line: Style,
    pub diff_divider_line: Style,
    pub diff_line_number: Style,
    
    // Chat styles
    pub chat_user_message: Style,
    pub chat_assistant_message: Style,
    pub chat_system_message: Style,
    pub chat_tool_message: Style,
    pub chat_timestamp: Style,
}

/// Markdown styling configuration
/// 
/// This structure provides styling for markdown rendering,
/// including syntax highlighting and formatting options.
#[derive(Debug, Clone)]
pub struct MarkdownStyles {
    pub document: Style,
    pub block_quote: Style,
    pub heading: Style,
    pub h1: Style,
    pub h2: Style,
    pub h3: Style,
    pub h4: Style,
    pub h5: Style,
    pub h6: Style,
    pub paragraph: Style,
    pub emphasis: Style,
    pub strong: Style,
    pub strikethrough: Style,
    pub link: Style,
    pub link_text: Style,
    pub image: Style,
    pub image_text: Style,
    pub code_inline: Style,
    pub code_block: Style,
    pub horizontal_rule: Style,
    pub list_item: Style,
    pub list_enumeration: Style,
    pub task_checked: Style,
    pub task_unchecked: Style,
    pub table: Style,
    pub table_header: Style,
    pub table_cell: Style,
}

/// Icon set for different UI elements
#[derive(Debug, Clone)]
pub struct IconSet {
    // Navigation icons
    pub folder_open: String,
    pub folder_closed: String,
    pub file: String,
    pub session: String,
    
    // Chat icons
    pub user: String,
    pub assistant: String,
    pub system: String,
    pub tool: String,
    pub attachment: String,
    
    // Status icons
    pub success: String,
    pub error: String,
    pub warning: String,
    pub info: String,
    pub loading: String,
    
    // Action icons
    pub copy: String,
    pub edit: String,
    pub delete: String,
    pub search: String,
    pub settings: String,
    pub help: String,
    
    // Arrows and indicators
    pub arrow_right: String,
    pub arrow_down: String,
    pub arrow_up: String,
    pub arrow_left: String,
    pub bullet: String,
    pub checkmark: String,
}

/// Animation configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    pub enabled: bool,
    pub duration_fast: u64,    // milliseconds
    pub duration_medium: u64,
    pub duration_slow: u64,
    pub easing: EasingType,
}

/// Easing types for animations
#[derive(Debug, Clone)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
}

impl Theme {
    /// Get styles, building them if necessary
    /// 
    /// This function builds and caches component styles based on the theme colors,
    /// similar to the Crush theme.buildStyles() method.
    pub fn styles(&mut self) -> &Styles {
        if self.styles.is_none() {
            self.styles = Some(self.build_styles());
        }
        self.styles.as_ref().unwrap()
    }
    
    /// Build styles from theme colors
    fn build_styles(&self) -> Styles {
        let base = Style::default().fg(self.fg_base);
        
        Styles {
            base,
            selected_base: base.bg(self.primary),
            
            // Typography
            title: base.fg(self.accent).add_modifier(Modifier::BOLD),
            subtitle: base.fg(self.secondary).add_modifier(Modifier::BOLD),
            text: base,
            text_selected: base.bg(self.primary).fg(self.fg_selected),
            muted: base.fg(self.fg_muted),
            subtle: base.fg(self.fg_subtle),
            
            // Status
            success: base.fg(self.success),
            error: base.fg(self.error),
            warning: base.fg(self.warning),
            info: base.fg(self.info),
            
            // Text input styles
            text_input_focused: base,
            text_input_blurred: base.fg(self.fg_muted),
            text_input_placeholder: base.fg(self.fg_subtle),
            text_input_prompt: base.fg(self.tertiary),
            text_input_cursor: base.fg(self.secondary),
            
            // Text area styles
            text_area_focused: base,
            text_area_blurred: base.fg(self.fg_muted),
            text_area_line_number: base.fg(self.fg_subtle),
            text_area_cursor_line: base,
            
            // Help styles
            help_short_key: base.fg(self.fg_muted),
            help_short_desc: base.fg(self.fg_subtle),
            help_short_separator: base.fg(self.border),
            help_ellipsis: base.fg(self.border),
            help_full_key: base.fg(self.fg_muted),
            help_full_desc: base.fg(self.fg_subtle),
            help_full_separator: base.fg(self.border),
            
            // Dialog styles
            dialog_border: base.fg(self.border_focus),
            dialog_title: base.fg(self.accent).add_modifier(Modifier::BOLD),
            dialog_content: base,
            
            // List styles
            list_item: base,
            list_item_selected: base.bg(self.primary).fg(self.fg_selected),
            list_item_focused: base.bg(self.bg_subtle),
            
            // File picker styles
            file_picker_cursor: base.fg(self.fg_base),
            file_picker_directory: base.fg(self.primary),
            file_picker_file: base.fg(self.fg_base),
            file_picker_symlink: base.fg(self.fg_subtle),
            file_picker_selected: base.bg(self.primary).fg(self.fg_base),
            file_picker_disabled: base.fg(self.fg_muted),
            file_picker_permission: base.fg(self.fg_muted),
            file_picker_size: base.fg(self.fg_muted),
            
            // Diff viewer styles
            diff_equal_line: base.fg(self.fg_muted).bg(self.bg_base),
            diff_insert_line: base.fg(self.green).bg(self.green_dark),
            diff_delete_line: base.fg(self.red).bg(self.red_dark),
            diff_divider_line: base.fg(self.fg_half_muted).bg(self.bg_base_lighter),
            diff_line_number: base.fg(self.fg_half_muted),
            
            // Chat styles
            chat_user_message: base.fg(self.blue),
            chat_assistant_message: base.fg(self.green),
            chat_system_message: base.fg(self.warning),
            chat_tool_message: base.fg(self.info),
            chat_timestamp: base.fg(self.fg_subtle),
        }
    }
}

/// Theme manager for handling multiple themes
/// 
/// This manager handles theme registration, switching, and provides
/// global access to the current theme, similar to Crush's theme manager.
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current: String,
}

impl ThemeManager {
    /// Create a new theme manager with default themes
    pub fn new() -> Self {
        let mut manager = Self {
            themes: HashMap::new(),
            current: "goofy_dark".to_string(),
        };
        
        // Load default themes
        manager.register_theme(presets::goofy_dark());
        manager.register_theme(presets::goofy_light());
        manager.register_theme(presets::classic_dark());
        manager.register_theme(presets::classic_light());
        manager.register_theme(presets::high_contrast());
        manager.register_theme(presets::monochrome());
        
        manager
    }
    
    /// Register a new theme
    pub fn register_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }
    
    /// Get the current theme (mutable reference for lazy style building)
    pub fn current_theme_mut(&mut self) -> &mut Theme {
        self.themes.get_mut(&self.current)
            .expect("Current theme should always exist")
    }
    
    /// Get the current theme (immutable reference)
    pub fn current_theme(&self) -> &Theme {
        self.themes.get(&self.current)
            .expect("Current theme should always exist")
    }
    
    /// Set the current theme
    pub fn set_theme(&mut self, name: &str) -> Result<(), String> {
        if self.themes.contains_key(name) {
            self.current = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }
    
    /// List available themes
    pub fn list_themes(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get theme by name
    pub fn get_theme(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

// Global theme manager instance
static mut GLOBAL_THEME_MANAGER: Option<ThemeManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get the global theme manager
pub fn theme_manager() -> &'static mut ThemeManager {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_THEME_MANAGER = Some(ThemeManager::new());
        });
        GLOBAL_THEME_MANAGER.as_mut().unwrap()
    }
}

/// Get the current theme (mutable reference for style building)
pub fn current_theme_mut() -> &'static mut Theme {
    theme_manager().current_theme_mut()
}

/// Get the current theme (immutable reference)
pub fn current_theme() -> &'static Theme {
    theme_manager().current_theme()
}

/// Set the current theme
pub fn set_current_theme(name: &str) -> Result<()> {
    theme_manager().set_theme(name)
        .map_err(|e| anyhow::anyhow!(e))
}

/// Get styles for the current theme (builds them if necessary)
pub fn current_styles() -> &'static Styles {
    current_theme_mut().styles()
}


/// Utility functions for theme operations
pub mod utils {
    use super::*;
    
    /// Blend two colors
    pub fn blend_colors(color1: Color, color2: Color, ratio: f32) -> Color {
        // Simple RGB blending - could be enhanced with HSL/HSV
        match (color1, color2) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 * (1.0 - ratio) + r2 as f32 * ratio) as u8;
                let g = (g1 as f32 * (1.0 - ratio) + g2 as f32 * ratio) as u8;
                let b = (b1 as f32 * (1.0 - ratio) + b2 as f32 * ratio) as u8;
                Color::Rgb(r, g, b)
            }
            _ => color1, // Fallback to first color for non-RGB colors
        }
    }
    
    /// Darken a color by a percentage
    pub fn darken_color(color: Color, percentage: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let factor = 1.0 - (percentage / 100.0);
                Color::Rgb(
                    (r as f32 * factor) as u8,
                    (g as f32 * factor) as u8,
                    (b as f32 * factor) as u8,
                )
            }
            _ => color,
        }
    }
    
    /// Lighten a color by a percentage
    pub fn lighten_color(color: Color, percentage: f32) -> Color {
        match color {
            Color::Rgb(r, g, b) => {
                let factor = percentage / 100.0;
                Color::Rgb(
                    ((r as f32 + (255.0 - r as f32) * factor) as u8).min(255),
                    ((g as f32 + (255.0 - g as f32) * factor) as u8).min(255),
                    ((b as f32 + (255.0 - b as f32) * factor) as u8).min(255),
                )
            }
            _ => color,
        }
    }
    
    /// Get contrasting text color for a background
    pub fn contrasting_text_color(bg_color: Color) -> Color {
        match bg_color {
            Color::Rgb(r, g, b) => {
                // Calculate luminance
                let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;
                if luminance > 0.5 {
                    Color::Black
                } else {
                    Color::White
                }
            }
            _ => Color::White,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert!(!manager.themes.is_empty());
        assert!(manager.themes.contains_key("goofy_dark"));
    }
    
    #[test]
    fn test_theme_switching() {
        let mut manager = ThemeManager::new();
        assert_eq!(manager.current, "goofy_dark");
        
        assert!(manager.set_theme("goofy_light").is_ok());
        assert_eq!(manager.current, "goofy_light");
        
        assert!(manager.set_theme("nonexistent").is_err());
    }
    
    #[test]
    fn test_color_blending() {
        let color1 = Color::Rgb(255, 0, 0);  // Red
        let color2 = Color::Rgb(0, 255, 0);  // Green
        let blended = utils::blend_colors(color1, color2, 0.5);
        
        if let Color::Rgb(r, g, b) = blended {
            assert_eq!(r, 127);  // Roughly half
            assert_eq!(g, 127);  // Roughly half
            assert_eq!(b, 0);
        } else {
            panic!("Expected RGB color");
        }
    }
}