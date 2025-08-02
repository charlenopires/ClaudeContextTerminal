//! Theme presets for the Goofy TUI
//! 
//! This module contains predefined themes that can be used out of the box.

use ratatui::style::{Color, Style, Modifier};
use super::{Theme, ColorScheme, StyleMap, IconSet, AnimationConfig, EasingType};

/// Goofy Dark Theme - Main dark theme
pub fn goofy_dark() -> Theme {
    let colors = ColorScheme {
        // Primary colors
        primary: Color::Rgb(130, 130, 255),     // Soft blue
        secondary: Color::Rgb(255, 130, 130),   // Soft red
        tertiary: Color::Rgb(130, 255, 130),    // Soft green
        accent: Color::Rgb(255, 255, 130),      // Soft yellow
        
        // Background colors
        bg_base: Color::Rgb(24, 24, 37),        // Very dark blue-gray
        bg_base_lighter: Color::Rgb(32, 32, 48), // Slightly lighter
        bg_subtle: Color::Rgb(40, 40, 58),      // Subtle highlight
        bg_overlay: Color::Rgb(48, 48, 68),     // Modal overlay
        bg_selected: Color::Rgb(65, 65, 100),   // Selected item
        
        // Foreground colors
        fg_base: Color::Rgb(220, 220, 230),     // Main text
        fg_muted: Color::Rgb(160, 160, 180),    // Muted text
        fg_half_muted: Color::Rgb(190, 190, 210), // Half muted
        fg_subtle: Color::Rgb(130, 130, 150),   // Subtle text
        fg_selected: Color::Rgb(255, 255, 255), // Selected text
        
        // Border colors
        border: Color::Rgb(80, 80, 100),        // Default border
        border_focus: Color::Rgb(130, 130, 255), // Focused border
        
        // Status colors
        success: Color::Rgb(100, 200, 100),     // Green
        error: Color::Rgb(220, 100, 100),       // Red
        warning: Color::Rgb(220, 180, 100),     // Orange
        info: Color::Rgb(100, 150, 220),        // Blue
        
        // Special colors
        white: Color::Rgb(255, 255, 255),
        blue_light: Color::Rgb(150, 180, 255),
        blue: Color::Rgb(100, 130, 255),
        yellow: Color::Rgb(255, 220, 100),
        green: Color::Rgb(100, 200, 100),
        green_dark: Color::Rgb(80, 160, 80),
        green_light: Color::Rgb(120, 220, 120),
        red: Color::Rgb(220, 100, 100),
        red_dark: Color::Rgb(180, 80, 80),
        red_light: Color::Rgb(240, 120, 120),
        cherry: Color::Rgb(200, 80, 120),
    };
    
    let styles = build_styles(&colors);
    
    Theme {
        name: "goofy_dark".to_string(),
        is_dark: true,
        colors,
        styles,
        icons: default_icons(),
        animations: AnimationConfig::default(),
    }
}

/// Goofy Light Theme - Main light theme
pub fn goofy_light() -> Theme {
    let colors = ColorScheme {
        // Primary colors
        primary: Color::Rgb(80, 80, 200),       // Darker blue
        secondary: Color::Rgb(200, 80, 80),     // Darker red
        tertiary: Color::Rgb(80, 180, 80),      // Darker green
        accent: Color::Rgb(200, 160, 80),       // Darker yellow
        
        // Background colors
        bg_base: Color::Rgb(250, 250, 255),     // Very light blue-white
        bg_base_lighter: Color::Rgb(245, 245, 250), // Slightly darker
        bg_subtle: Color::Rgb(240, 240, 248),   // Subtle highlight
        bg_overlay: Color::Rgb(230, 230, 240),  // Modal overlay
        bg_selected: Color::Rgb(220, 220, 235), // Selected item
        
        // Foreground colors
        fg_base: Color::Rgb(40, 40, 50),        // Main text
        fg_muted: Color::Rgb(80, 80, 100),      // Muted text
        fg_half_muted: Color::Rgb(60, 60, 80),  // Half muted
        fg_subtle: Color::Rgb(120, 120, 140),   // Subtle text
        fg_selected: Color::Rgb(20, 20, 30),    // Selected text
        
        // Border colors
        border: Color::Rgb(180, 180, 200),      // Default border
        border_focus: Color::Rgb(80, 80, 200),  // Focused border
        
        // Status colors
        success: Color::Rgb(60, 150, 60),       // Green
        error: Color::Rgb(180, 60, 60),         // Red
        warning: Color::Rgb(180, 120, 60),      // Orange
        info: Color::Rgb(60, 100, 180),         // Blue
        
        // Special colors
        white: Color::Rgb(255, 255, 255),
        blue_light: Color::Rgb(120, 150, 255),
        blue: Color::Rgb(80, 120, 220),
        yellow: Color::Rgb(200, 160, 60),
        green: Color::Rgb(80, 160, 80),
        green_dark: Color::Rgb(60, 120, 60),
        green_light: Color::Rgb(100, 180, 100),
        red: Color::Rgb(180, 60, 60),
        red_dark: Color::Rgb(140, 40, 40),
        red_light: Color::Rgb(200, 80, 80),
        cherry: Color::Rgb(160, 60, 100),
    };
    
    let styles = build_styles(&colors);
    
    Theme {
        name: "goofy_light".to_string(),
        is_dark: false,
        colors,
        styles,
        icons: default_icons(),
        animations: AnimationConfig::default(),
    }
}

/// Classic Dark Theme - Traditional terminal colors
pub fn classic_dark() -> Theme {
    let colors = ColorScheme {
        // Primary colors
        primary: Color::Blue,
        secondary: Color::Cyan,
        tertiary: Color::Green,
        accent: Color::Yellow,
        
        // Background colors
        bg_base: Color::Black,
        bg_base_lighter: Color::Rgb(20, 20, 20),
        bg_subtle: Color::Rgb(40, 40, 40),
        bg_overlay: Color::Rgb(60, 60, 60),
        bg_selected: Color::Rgb(80, 80, 80),
        
        // Foreground colors
        fg_base: Color::White,
        fg_muted: Color::Gray,
        fg_half_muted: Color::Rgb(180, 180, 180),
        fg_subtle: Color::DarkGray,
        fg_selected: Color::White,
        
        // Border colors
        border: Color::Gray,
        border_focus: Color::Blue,
        
        // Status colors
        success: Color::Green,
        error: Color::Red,
        warning: Color::Yellow,
        info: Color::Cyan,
        
        // Special colors
        white: Color::White,
        blue_light: Color::LightBlue,
        blue: Color::Blue,
        yellow: Color::Yellow,
        green: Color::Green,
        green_dark: Color::DarkGray,
        green_light: Color::LightGreen,
        red: Color::Red,
        red_dark: Color::Rgb(139, 0, 0),
        red_light: Color::LightRed,
        cherry: Color::Magenta,
    };
    
    let styles = build_styles(&colors);
    
    Theme {
        name: "classic_dark".to_string(),
        is_dark: true,
        colors,
        styles,
        icons: ascii_icons(),
        animations: AnimationConfig {
            enabled: false,
            ..Default::default()
        },
    }
}

/// Classic Light Theme - Traditional light terminal colors
pub fn classic_light() -> Theme {
    let colors = ColorScheme {
        // Primary colors
        primary: Color::Blue,
        secondary: Color::Red,
        tertiary: Color::Green,
        accent: Color::Rgb(180, 120, 0), // Dark yellow
        
        // Background colors
        bg_base: Color::White,
        bg_base_lighter: Color::Rgb(248, 248, 248),
        bg_subtle: Color::Rgb(240, 240, 240),
        bg_overlay: Color::Rgb(220, 220, 220),
        bg_selected: Color::Rgb(200, 200, 200),
        
        // Foreground colors
        fg_base: Color::Black,
        fg_muted: Color::DarkGray,
        fg_half_muted: Color::Gray,
        fg_subtle: Color::LightBlue,
        fg_selected: Color::Black,
        
        // Border colors
        border: Color::Gray,
        border_focus: Color::Blue,
        
        // Status colors
        success: Color::Green,
        error: Color::Red,
        warning: Color::Rgb(180, 120, 0),
        info: Color::Blue,
        
        // Special colors
        white: Color::White,
        blue_light: Color::LightBlue,
        blue: Color::Blue,
        yellow: Color::Rgb(180, 120, 0),
        green: Color::Green,
        green_dark: Color::Rgb(0, 100, 0),
        green_light: Color::LightGreen,
        red: Color::Red,
        red_dark: Color::Rgb(139, 0, 0),
        red_light: Color::LightRed,
        cherry: Color::Magenta,
    };
    
    let styles = build_styles(&colors);
    
    Theme {
        name: "classic_light".to_string(),
        is_dark: false,
        colors,
        styles,
        icons: ascii_icons(),
        animations: AnimationConfig {
            enabled: false,
            ..Default::default()
        },
    }
}

/// Build style map from color scheme
fn build_styles(colors: &ColorScheme) -> StyleMap {
    let base = Style::default().fg(colors.fg_base);
    
    StyleMap {
        base,
        selected_base: base.bg(colors.primary),
        title: base.fg(colors.accent).add_modifier(Modifier::BOLD),
        subtitle: base.fg(colors.secondary).add_modifier(Modifier::BOLD),
        text: base,
        text_selected: base.bg(colors.primary).fg(colors.fg_selected),
        muted: base.fg(colors.fg_muted),
        subtle: base.fg(colors.fg_subtle),
        success: base.fg(colors.success),
        error: base.fg(colors.error),
        warning: base.fg(colors.warning),
        info: base.fg(colors.info),
        
        // Component-specific styles
        chat_message: base.bg(colors.bg_subtle),
        chat_user: base.fg(colors.primary),
        chat_assistant: base.fg(colors.secondary),
        chat_system: base.fg(colors.fg_muted),
        chat_tool: base.fg(colors.tertiary),
        
        sidebar_item: base.fg(colors.fg_base),
        sidebar_selected: base.bg(colors.bg_selected).fg(colors.fg_selected),
        sidebar_expanded: base.fg(colors.accent),
        
        dialog_background: base.bg(colors.bg_overlay),
        dialog_border: base.fg(colors.border_focus),
        dialog_title: base.fg(colors.accent).add_modifier(Modifier::BOLD),
        
        editor_line_number: base.fg(colors.fg_subtle),
        editor_cursor: base.bg(colors.secondary),
        editor_selection: base.bg(colors.bg_selected),
        
        status_bar: base.bg(colors.bg_base_lighter),
        status_info: base.fg(colors.info),
        status_error: base.fg(colors.error),
    }
}

/// Default Unicode icons
pub fn default_icons() -> IconSet {
    IconSet {
        // Navigation icons
        folder_open: "📂".to_string(),
        folder_closed: "📁".to_string(),
        file: "📄".to_string(),
        session: "💬".to_string(),
        
        // Chat icons
        user: "👤".to_string(),
        assistant: "🤖".to_string(),
        system: "⚙️".to_string(),
        tool: "🔧".to_string(),
        attachment: "📎".to_string(),
        
        // Status icons
        success: "✅".to_string(),
        error: "❌".to_string(),
        warning: "⚠️".to_string(),
        info: "ℹ️".to_string(),
        loading: "⏳".to_string(),
        
        // Action icons
        copy: "📋".to_string(),
        edit: "✏️".to_string(),
        delete: "🗑️".to_string(),
        search: "🔍".to_string(),
        settings: "⚙️".to_string(),
        help: "❓".to_string(),
        
        // Arrows and indicators
        arrow_right: "▶".to_string(),
        arrow_down: "▼".to_string(),
        arrow_up: "▲".to_string(),
        arrow_left: "◀".to_string(),
        bullet: "•".to_string(),
        checkmark: "✓".to_string(),
    }
}

/// ASCII-only icons for terminals without Unicode support
pub fn ascii_icons() -> IconSet {
    IconSet {
        // Navigation icons
        folder_open: "[+]".to_string(),
        folder_closed: "[-]".to_string(),
        file: "   ".to_string(),
        session: "[S]".to_string(),
        
        // Chat icons
        user: "[U]".to_string(),
        assistant: "[A]".to_string(),
        system: "[*]".to_string(),
        tool: "[T]".to_string(),
        attachment: "[@]".to_string(),
        
        // Status icons
        success: "[OK]".to_string(),
        error: "[ERROR]".to_string(),
        warning: "[WARN]".to_string(),
        info: "[INFO]".to_string(),
        loading: "[...]".to_string(),
        
        // Action icons
        copy: "[C]".to_string(),
        edit: "[E]".to_string(),
        delete: "[D]".to_string(),
        search: "[?]".to_string(),
        settings: "[CFG]".to_string(),
        help: "[H]".to_string(),
        
        // Arrows and indicators
        arrow_right: ">".to_string(),
        arrow_down: "v".to_string(),
        arrow_up: "^".to_string(),
        arrow_left: "<".to_string(),
        bullet: "*".to_string(),
        checkmark: "x".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_themes_have_required_fields() {
        let themes = vec![
            goofy_dark(),
            goofy_light(),
            classic_dark(),
            classic_light(),
        ];
        
        for theme in themes {
            assert!(!theme.name.is_empty());
            // Basic smoke test - ensure all colors are defined
            assert_ne!(theme.colors.primary, Color::Reset);
            assert_ne!(theme.colors.bg_base, Color::Reset);
            assert_ne!(theme.colors.fg_base, Color::Reset);
        }
    }
    
    #[test]
    fn test_icon_sets_complete() {
        let default_icons = default_icons();
        let ascii_icons = ascii_icons();
        
        // Ensure both icon sets have all required icons
        assert!(!default_icons.folder_open.is_empty());
        assert!(!default_icons.user.is_empty());
        assert!(!default_icons.success.is_empty());
        
        assert!(!ascii_icons.folder_open.is_empty());
        assert!(!ascii_icons.user.is_empty());
        assert!(!ascii_icons.success.is_empty());
    }
}