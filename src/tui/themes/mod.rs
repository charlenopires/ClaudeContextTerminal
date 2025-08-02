//! Advanced theming system for Goofy TUI
//! 
//! This module provides a comprehensive theming system with support for
//! color schemes, styles, animations, and responsive design.

use std::collections::HashMap;
use ratatui::style::{Color, Style, Modifier};
use serde::{Deserialize, Serialize};

pub mod colors;
pub mod styles;
pub mod presets;

/// Theme represents a complete visual style configuration
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub colors: ColorScheme,
    pub styles: StyleMap,
    pub icons: IconSet,
    pub animations: AnimationConfig,
}

/// Color scheme defining all colors used in the interface
#[derive(Debug, Clone)]
pub struct ColorScheme {
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub accent: Color,
    
    // Background colors
    pub bg_base: Color,
    pub bg_base_lighter: Color,
    pub bg_subtle: Color,
    pub bg_overlay: Color,
    pub bg_selected: Color,
    
    // Foreground colors
    pub fg_base: Color,
    pub fg_muted: Color,
    pub fg_half_muted: Color,
    pub fg_subtle: Color,
    pub fg_selected: Color,
    
    // Border colors
    pub border: Color,
    pub border_focus: Color,
    
    // Status colors
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    
    // Special colors
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
}

/// Style mapping for different UI elements
#[derive(Debug, Clone)]
pub struct StyleMap {
    pub base: Style,
    pub selected_base: Style,
    pub title: Style,
    pub subtitle: Style,
    pub text: Style,
    pub text_selected: Style,
    pub muted: Style,
    pub subtle: Style,
    pub success: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    
    // Component-specific styles
    pub chat_message: Style,
    pub chat_user: Style,
    pub chat_assistant: Style,
    pub chat_system: Style,
    pub chat_tool: Style,
    
    pub sidebar_item: Style,
    pub sidebar_selected: Style,
    pub sidebar_expanded: Style,
    
    pub dialog_background: Style,
    pub dialog_border: Style,
    pub dialog_title: Style,
    
    pub editor_line_number: Style,
    pub editor_cursor: Style,
    pub editor_selection: Style,
    
    pub status_bar: Style,
    pub status_info: Style,
    pub status_error: Style,
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

/// Theme manager for handling multiple themes
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
        
        manager
    }
    
    /// Register a new theme
    pub fn register_theme(&mut self, theme: Theme) {
        self.themes.insert(theme.name.clone(), theme);
    }
    
    /// Get the current theme
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

impl Default for ColorScheme {
    fn default() -> Self {
        presets::goofy_dark().colors
    }
}

impl Default for StyleMap {
    fn default() -> Self {
        presets::goofy_dark().styles
    }
}

impl Default for IconSet {
    fn default() -> Self {
        presets::default_icons()
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_fast: 150,
            duration_medium: 300,
            duration_slow: 500,
            easing: EasingType::EaseInOut,
        }
    }
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