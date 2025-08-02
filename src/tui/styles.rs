use ratatui::style::{Color, Modifier, Style};

/// Application theme configuration
#[derive(Debug, Clone)]
pub struct Theme {
    /// Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    /// Text colors
    pub text: Color,
    pub text_dim: Color,
    pub text_bright: Color,
    
    /// Background colors
    pub background: Color,
    pub background_alt: Color,
    
    /// Border colors
    pub border: Color,
    pub border_focused: Color,
    
    /// Status colors
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    /// Special colors
    pub placeholder: Color,
    pub selection: Color,
    pub cursor: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            primary: Color::Rgb(147, 51, 234),    // Purple
            secondary: Color::Rgb(59, 130, 246),  // Blue
            accent: Color::Rgb(236, 72, 153),     // Pink
            
            text: Color::Rgb(248, 250, 252),      // Slate-50
            text_dim: Color::Rgb(148, 163, 184),  // Slate-400
            text_bright: Color::Rgb(255, 255, 255), // White
            
            background: Color::Rgb(15, 23, 42),   // Slate-900
            background_alt: Color::Rgb(30, 41, 59), // Slate-800
            
            border: Color::Rgb(71, 85, 105),      // Slate-600
            border_focused: Color::Rgb(147, 51, 234), // Purple
            
            success: Color::Rgb(34, 197, 94),     // Green-500
            warning: Color::Rgb(245, 158, 11),    // Amber-500
            error: Color::Rgb(239, 68, 68),       // Red-500
            info: Color::Rgb(59, 130, 246),       // Blue-500
            
            placeholder: Color::Rgb(100, 116, 139), // Slate-500
            selection: Color::Rgb(30, 58, 138),   // Blue-900
            cursor: Color::Rgb(248, 250, 252),    // Slate-50
        }
    }
    
    /// Base style for normal elements
    pub fn base_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.background)
    }
    
    /// Style for text content
    pub fn text_style(&self) -> Style {
        Style::default().fg(self.text)
    }
    
    /// Style for borders
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
    
    /// Style for focused borders
    pub fn focused_border_style(&self) -> Style {
        Style::default()
            .fg(self.border_focused)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Style for selected items
    pub fn selection_style(&self) -> Style {
        Style::default()
            .bg(self.selection)
            .fg(self.text_bright)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Style for the status bar
    pub fn status_bar_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.background_alt)
    }
    
    /// Style for help text
    pub fn help_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .bg(self.background)
            .add_modifier(Modifier::BOLD)
    }
    
    /// Style for placeholder text
    pub fn placeholder_style(&self) -> Style {
        Style::default()
            .fg(self.placeholder)
            .add_modifier(Modifier::ITALIC)
    }
}