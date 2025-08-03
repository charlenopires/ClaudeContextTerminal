//! Style utilities and component-specific styling

use ratatui::style::{Color, Style, Modifier};
use super::Theme;

/// Style builder for creating consistent styles
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    /// Create a new style builder
    pub fn new() -> Self {
        Self {
            style: Style::default(),
        }
    }
    
    /// Create from existing style
    pub fn from_style(style: Style) -> Self {
        Self { style }
    }
    
    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.style = self.style.fg(color);
        self
    }
    
    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.style = self.style.bg(color);
        self
    }
    
    /// Add modifier
    pub fn modifier(mut self, modifier: Modifier) -> Self {
        self.style = self.style.add_modifier(modifier);
        self
    }
    
    /// Add bold
    pub fn bold(self) -> Self {
        self.modifier(Modifier::BOLD)
    }
    
    /// Add italic
    pub fn italic(self) -> Self {
        self.modifier(Modifier::ITALIC)
    }
    
    /// Add underline
    pub fn underline(self) -> Self {
        self.modifier(Modifier::UNDERLINED)
    }
    
    /// Add dim
    pub fn dim(self) -> Self {
        self.modifier(Modifier::DIM)
    }
    
    /// Add crossed out
    pub fn crossed_out(self) -> Self {
        self.modifier(Modifier::CROSSED_OUT)
    }
    
    /// Build the final style
    pub fn build(self) -> Style {
        self.style
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset style combinations for common UI patterns
pub struct StylePresets;

impl StylePresets {
    /// Create a button style
    pub fn button(theme: &Theme, focused: bool) -> Style {
        let colors = &theme.colors;
        if focused {
            StyleBuilder::new()
                .fg(colors.fg_selected)
                .bg(colors.primary)
                .bold()
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.fg_base)
                .bg(colors.bg_subtle)
                .build()
        }
    }
    
    /// Create an input field style
    pub fn input_field(theme: &Theme, focused: bool, error: bool) -> Style {
        let colors = &theme.colors;
        let mut builder = StyleBuilder::new().fg(colors.fg_base);
        
        if error {
            builder = builder.bg(colors.error).fg(colors.white);
        } else if focused {
            builder = builder.bg(colors.bg_base_lighter);
        } else {
            builder = builder.bg(colors.bg_subtle);
        }
        
        builder.build()
    }
    
    /// Create a list item style
    pub fn list_item(theme: &Theme, selected: bool, focused: bool) -> Style {
        let colors = &theme.colors;
        if selected && focused {
            StyleBuilder::new()
                .fg(colors.fg_selected)
                .bg(colors.primary)
                .build()
        } else if selected {
            StyleBuilder::new()
                .fg(colors.fg_base)
                .bg(colors.bg_selected)
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.fg_base)
                .build()
        }
    }
    
    /// Create a tab style
    pub fn tab(theme: &Theme, active: bool) -> Style {
        let colors = &theme.colors;
        if active {
            StyleBuilder::new()
                .fg(colors.accent)
                .bg(colors.bg_base)
                .bold()
                .underline()
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.fg_muted)
                .bg(colors.bg_subtle)
                .build()
        }
    }
    
    /// Create a badge style
    pub fn badge(theme: &Theme, badge_type: BadgeType) -> Style {
        let colors = &theme.colors;
        match badge_type {
            BadgeType::Success => StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.success)
                .bold()
                .build(),
            BadgeType::Error => StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.error)
                .bold()
                .build(),
            BadgeType::Warning => StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.warning)
                .bold()
                .build(),
            BadgeType::Info => StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.info)
                .build(),
            BadgeType::Default => StyleBuilder::new()
                .fg(colors.fg_base)
                .bg(colors.bg_subtle)
                .build(),
        }
    }
    
    /// Create a progress bar style
    pub fn progress_bar(theme: &Theme, completed: bool) -> Style {
        let colors = &theme.colors;
        if completed {
            StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.success)
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.white)
                .bg(colors.primary)
                .build()
        }
    }
    
    /// Create a border style
    pub fn border(theme: &Theme, focused: bool) -> Style {
        let colors = &theme.colors;
        if focused {
            StyleBuilder::new()
                .fg(colors.border_focus)
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.border)
                .build()
        }
    }
    
    /// Create a code block style
    pub fn code_block(theme: &Theme) -> Style {
        let colors = &theme.colors;
        StyleBuilder::new()
            .fg(colors.fg_base)
            .bg(colors.bg_base_lighter)
            .build()
    }
    
    /// Create an inline code style
    pub fn inline_code(theme: &Theme) -> Style {
        let colors = &theme.colors;
        StyleBuilder::new()
            .fg(colors.accent)
            .bg(colors.bg_subtle)
            .build()
    }
    
    /// Create a link style
    pub fn link(theme: &Theme, visited: bool) -> Style {
        let colors = &theme.colors;
        if visited {
            StyleBuilder::new()
                .fg(colors.secondary)
                .underline()
                .build()
        } else {
            StyleBuilder::new()
                .fg(colors.primary)
                .underline()
                .build()
        }
    }
    
    /// Create a tooltip style
    pub fn tooltip(theme: &Theme) -> Style {
        let colors = &theme.colors;
        StyleBuilder::new()
            .fg(colors.fg_base)
            .bg(colors.bg_overlay)
            .build()
    }
}

/// Badge types for different styling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeType {
    Success,
    Error,
    Warning,
    Info,
    Default,
}

/// Gradient utilities for creating smooth color transitions
pub struct Gradient {
    start: Color,
    end: Color,
    steps: usize,
}

impl Gradient {
    /// Create a new gradient
    pub fn new(start: Color, end: Color, steps: usize) -> Self {
        Self { start, end, steps }
    }
    
    /// Generate all colors in the gradient
    pub fn colors(&self) -> Vec<Color> {
        if self.steps == 0 {
            return vec![];
        }
        
        if self.steps == 1 {
            return vec![self.start];
        }
        
        let mut colors = Vec::with_capacity(self.steps);
        
        match (self.start, self.end) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                for i in 0..self.steps {
                    let ratio = i as f32 / (self.steps - 1) as f32;
                    let r = (r1 as f32 * (1.0 - ratio) + r2 as f32 * ratio) as u8;
                    let g = (g1 as f32 * (1.0 - ratio) + g2 as f32 * ratio) as u8;
                    let b = (b1 as f32 * (1.0 - ratio) + b2 as f32 * ratio) as u8;
                    colors.push(Color::Rgb(r, g, b));
                }
            }
            _ => {
                // Fallback for non-RGB colors
                for i in 0..self.steps {
                    if i < self.steps / 2 {
                        colors.push(self.start);
                    } else {
                        colors.push(self.end);
                    }
                }
            }
        }
        
        colors
    }
    
    /// Get color at specific position (0.0 - 1.0)
    pub fn color_at(&self, position: f32) -> Color {
        let position = position.clamp(0.0, 1.0);
        
        match (self.start, self.end) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                let r = (r1 as f32 * (1.0 - position) + r2 as f32 * position) as u8;
                let g = (g1 as f32 * (1.0 - position) + g2 as f32 * position) as u8;
                let b = (b1 as f32 * (1.0 - position) + b2 as f32 * position) as u8;
                Color::Rgb(r, g, b)
            }
            _ => if position < 0.5 { self.start } else { self.end },
        }
    }
}

/// Text styling utilities
pub struct TextStyler;

impl TextStyler {
    /// Apply syntax highlighting colors based on token type
    pub fn syntax_highlight(theme: &Theme, token_type: SyntaxTokenType) -> Style {
        let colors = &theme.colors;
        match token_type {
            SyntaxTokenType::Keyword => StyleBuilder::new()
                .fg(colors.blue)
                .bold()
                .build(),
            SyntaxTokenType::String => StyleBuilder::new()
                .fg(colors.green)
                .build(),
            SyntaxTokenType::Number => StyleBuilder::new()
                .fg(colors.yellow)
                .build(),
            SyntaxTokenType::Comment => StyleBuilder::new()
                .fg(colors.fg_muted)
                .italic()
                .build(),
            SyntaxTokenType::Function => StyleBuilder::new()
                .fg(colors.secondary)
                .build(),
            SyntaxTokenType::Type => StyleBuilder::new()
                .fg(colors.tertiary)
                .build(),
            SyntaxTokenType::Variable => StyleBuilder::new()
                .fg(colors.fg_base)
                .build(),
            SyntaxTokenType::Operator => StyleBuilder::new()
                .fg(colors.accent)
                .build(),
            SyntaxTokenType::Bracket => StyleBuilder::new()
                .fg(colors.fg_half_muted)
                .build(),
            SyntaxTokenType::Error => StyleBuilder::new()
                .fg(colors.error)
                .underline()
                .build(),
        }
    }
    
    /// Apply diff highlighting
    pub fn diff_highlight(theme: &Theme, diff_type: DiffType) -> Style {
        let colors = &theme.colors;
        match diff_type {
            DiffType::Added => StyleBuilder::new()
                .fg(colors.green)
                .bg(colors.green_dark)
                .build(),
            DiffType::Removed => StyleBuilder::new()
                .fg(colors.red)
                .bg(colors.red_dark)
                .build(),
            DiffType::Modified => StyleBuilder::new()
                .fg(colors.yellow)
                .bg(colors.warning)
                .build(),
            DiffType::Context => StyleBuilder::new()
                .fg(colors.fg_muted)
                .build(),
        }
    }
    
    /// Apply emphasis styling
    pub fn emphasis(theme: &Theme, emphasis_type: EmphasisType) -> Style {
        let colors = &theme.colors;
        match emphasis_type {
            EmphasisType::Strong => StyleBuilder::new()
                .fg(colors.fg_base)
                .bold()
                .build(),
            EmphasisType::Emphasis => StyleBuilder::new()
                .fg(colors.accent)
                .italic()
                .build(),
            EmphasisType::Subtle => StyleBuilder::new()
                .fg(colors.fg_subtle)
                .build(),
            EmphasisType::Highlight => StyleBuilder::new()
                .fg(colors.fg_base)
                .bg(colors.warning)
                .build(),
        }
    }
}

/// Syntax token types for highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxTokenType {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Bracket,
    Error,
}

/// Diff types for version control highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffType {
    Added,
    Removed,
    Modified,
    Context,
}

/// Text emphasis types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmphasisType {
    Strong,
    Emphasis,
    Subtle,
    Highlight,
}

/// Animation-aware styling
pub struct AnimatedStyle {
    pub from: Style,
    pub to: Style,
    pub progress: f32, // 0.0 - 1.0
}

impl AnimatedStyle {
    /// Create a new animated style
    pub fn new(from: Style, to: Style) -> Self {
        Self {
            from,
            to,
            progress: 0.0,
        }
    }
    
    /// Update animation progress
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
    
    /// Get current interpolated style
    pub fn current_style(&self) -> Style {
        if self.progress <= 0.0 {
            return self.from;
        }
        if self.progress >= 1.0 {
            return self.to;
        }
        
        // Interpolate between styles
        // This is a simplified version - a full implementation would
        // interpolate all style properties
        let mut style = self.from;
        
        // Interpolate foreground color if both styles have RGB colors
        if let (Some(Color::Rgb(r1, g1, b1)), Some(Color::Rgb(r2, g2, b2))) = 
            (self.from.fg, self.to.fg) {
            let r = (r1 as f32 * (1.0 - self.progress) + r2 as f32 * self.progress) as u8;
            let g = (g1 as f32 * (1.0 - self.progress) + g2 as f32 * self.progress) as u8;
            let b = (b1 as f32 * (1.0 - self.progress) + b2 as f32 * self.progress) as u8;
            style = style.fg(Color::Rgb(r, g, b));
        }
        
        // Interpolate background color if both styles have RGB colors
        if let (Some(Color::Rgb(r1, g1, b1)), Some(Color::Rgb(r2, g2, b2))) = 
            (self.from.bg, self.to.bg) {
            let r = (r1 as f32 * (1.0 - self.progress) + r2 as f32 * self.progress) as u8;
            let g = (g1 as f32 * (1.0 - self.progress) + g2 as f32 * self.progress) as u8;
            let b = (b1 as f32 * (1.0 - self.progress) + b2 as f32 * self.progress) as u8;
            style = style.bg(Color::Rgb(r, g, b));
        }
        
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::themes::presets;
    
    #[test]
    fn test_style_builder() {
        let style = StyleBuilder::new()
            .fg(Color::Red)
            .bg(Color::Blue)
            .bold()
            .italic()
            .build();
        
        assert_eq!(style.fg, Some(Color::Red));
        assert_eq!(style.bg, Some(Color::Blue));
        assert!(style.add_modifier.contains(Modifier::BOLD));
        assert!(style.add_modifier.contains(Modifier::ITALIC));
    }
    
    #[test]
    fn test_gradient_generation() {
        let gradient = Gradient::new(
            Color::Rgb(255, 0, 0),  // Red
            Color::Rgb(0, 255, 0),  // Green
            3
        );
        
        let colors = gradient.colors();
        assert_eq!(colors.len(), 3);
        
        // First color should be red
        assert_eq!(colors[0], Color::Rgb(255, 0, 0));
        // Last color should be green
        assert_eq!(colors[2], Color::Rgb(0, 255, 0));
        // Middle should be a mix
        if let Color::Rgb(r, g, b) = colors[1] {
            assert!(r > 0 && r < 255);
            assert!(g > 0 && g < 255);
            assert_eq!(b, 0);
        }
    }
    
    #[test]
    fn test_animated_style() {
        let from = StyleBuilder::new().fg(Color::Red).build();
        let to = StyleBuilder::new().fg(Color::Blue).build();
        
        let mut animated = AnimatedStyle::new(from, to);
        
        animated.update_progress(0.0);
        let current = animated.current_style();
        assert_eq!(current.fg, Some(Color::Red));
        
        animated.update_progress(1.0);
        let current = animated.current_style();
        assert_eq!(current.fg, Some(Color::Blue));
    }
    
    #[test]
    fn test_style_presets() {
        let theme = presets::goofy_dark();
        
        let button_focused = StylePresets::button(&theme, true);
        let button_normal = StylePresets::button(&theme, false);
        
        // Focused button should have different styling
        assert_ne!(button_focused.bg, button_normal.bg);
        
        let list_selected = StylePresets::list_item(&theme, true, true);
        let list_normal = StylePresets::list_item(&theme, false, false);
        
        // Selected item should have different styling
        assert_ne!(list_selected.bg, list_normal.bg);
    }
}