//! Splash screen component for Goofy TUI
//! 
//! Displays the Goofy logo, version, and welcome information when the application starts.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::tui::{
    components::{Component, ComponentState, logo::{render_logo, render_small_logo, LogoOpts}},
    themes::Theme,
    themes::colors::ColorPalette,
};
use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use anyhow::Result;

/// Splash screen component showing Goofy branding and information
pub struct SplashComponent {
    state: ComponentState,
    version: String,
    show_info: bool,
    compact_mode: bool,
}

impl SplashComponent {
    /// Create a new splash component
    pub fn new(version: String) -> Self {
        Self {
            state: ComponentState::new(),
            version,
            show_info: true,
            compact_mode: false,
        }
    }
    
    /// Set whether to show additional info below the logo
    pub fn with_info(mut self, show_info: bool) -> Self {
        self.show_info = show_info;
        self
    }
    
    /// Set compact mode for smaller screens
    pub fn with_compact_mode(mut self, compact: bool) -> Self {
        self.compact_mode = compact;
        self
    }
    
    /// Render the logo section
    fn render_logo(&self, area: Rect, theme: &Theme) -> Text<'static> {
        let opts = LogoOpts {
            gradient_start: ColorPalette::GOOFY_ORANGE,
            gradient_end: ColorPalette::GOOFY_PURPLE,
            field_color: ColorPalette::GOOFY_ORANGE,
            brand_color: ColorPalette::GOOFY_PURPLE,
            version_color: ColorPalette::GOOFY_ORANGE,
            width: area.width as usize,
            compact: self.compact_mode || area.width < 60 || area.height < 15,
        };
        
        if opts.compact {
            let small_logo = render_small_logo(area.width as usize, opts);
            Text::from(vec![small_logo])
        } else {
            render_logo(&self.version, opts)
        }
    }
    
    /// Create info section with project details
    fn create_info_section(&self, theme: &Theme) -> Text<'static> {
        let info_lines = vec![
            Line::from(vec![
                Span::styled("🚀 ", Style::default().fg(ColorPalette::GOOFY_ORANGE)),
                Span::styled("Welcome to Goofy", Style::default().fg(theme.text)),
                Span::styled(" - Your AI Coding Assistant", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📚 ", Style::default().fg(ColorPalette::GOOFY_BLUE)),
                Span::styled("Features:", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Multi-provider LLM support", Style::default().fg(theme.text_dim)),
                Span::styled(" (OpenAI, Anthropic, Ollama)", Style::default().fg(theme.placeholder)),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Interactive Terminal UI", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Session management and persistence", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::raw("  • "),
                Span::styled("Comprehensive tool system", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚡ ", Style::default().fg(ColorPalette::GOOFY_YELLOW)),
                Span::styled("Quick Start:", Style::default().fg(theme.text)),
            ]),
            Line::from(vec![
                Span::raw("  • Press "),
                Span::styled("Ctrl+N", Style::default().fg(ColorPalette::GOOFY_ORANGE)),
                Span::styled(" to create a new session", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::raw("  • Press "),
                Span::styled("Ctrl+P", Style::default().fg(ColorPalette::GOOFY_ORANGE)),
                Span::styled(" to open command palette", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(vec![
                Span::raw("  • Press "),
                Span::styled("?", Style::default().fg(ColorPalette::GOOFY_ORANGE)),
                Span::styled(" for help", Style::default().fg(theme.text_dim)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🔧 ", Style::default().fg(ColorPalette::GOOFY_GREEN)),
                Span::styled("Status: ", Style::default().fg(theme.text)),
                Span::styled("Ready", Style::default().fg(ColorPalette::SUCCESS_GREEN)),
            ]),
        ];
        
        Text::from(info_lines)
    }
    
    /// Check if the screen is too small for full display
    fn is_small_screen(&self) -> bool {
        self.state.size.width < 60 || self.state.size.height < 20
    }
}

#[async_trait]
impl Component for SplashComponent {
    async fn handle_key_event(&mut self, _event: KeyEvent) -> Result<()> {
        // Splash screen is typically read-only, but could handle navigation
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, _event: MouseEvent) -> Result<()> {
        Ok(())
    }
    
    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Update our size
        self.state.size = area;
        
        // Determine layout based on screen size and content
        let is_small = self.is_small_screen();
        
        if is_small {
            // Compact layout for small screens
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Logo
                    Constraint::Min(1),    // Info (if shown)
                ])
                .split(area);
            
            // Render compact logo
            let logo_text = self.render_logo(chunks[0], theme);
            let logo_paragraph = Paragraph::new(logo_text)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            frame.render_widget(logo_paragraph, chunks[0]);
            
            // Render minimal info if requested
            if self.show_info {
                let info_text = Text::from(vec![
                    Line::from(vec![
                        Span::styled("Goofy AI Assistant ", Style::default().fg(theme.text)),
                        Span::styled(&self.version, Style::default().fg(theme.text_dim)),
                    ]),
                    Line::from(vec![
                        Span::styled("Press ", Style::default().fg(theme.text_dim)),
                        Span::styled("?", Style::default().fg(ColorPalette::GOOFY_ORANGE)),
                        Span::styled(" for help", Style::default().fg(theme.text_dim)),
                    ]),
                ]);
                
                let info_paragraph = Paragraph::new(info_text)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true });
                frame.render_widget(info_paragraph, chunks[1]);
            }
        } else {
            // Full layout for larger screens
            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Logo area
                    Constraint::Length(2),  // Spacing
                    Constraint::Min(1),     // Info area
                ])
                .split(area);
            
            // Render full logo
            let logo_text = self.render_logo(main_chunks[0], theme);
            let logo_paragraph = Paragraph::new(logo_text)
                .alignment(Alignment::Center);
            frame.render_widget(logo_paragraph, main_chunks[0]);
            
            // Render info section if requested
            if self.show_info {
                let info_text = self.create_info_section(theme);
                let info_paragraph = Paragraph::new(info_text)
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(ColorPalette::GOOFY_ORANGE))
                            .title(" Welcome ")
                            .title_style(Style::default().fg(ColorPalette::GOOFY_PURPLE))
                    );
                
                // Center the info block
                let info_area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ])
                    .split(main_chunks[2])[1];
                
                frame.render_widget(info_paragraph, info_area);
            }
        }
    }
    
    fn size(&self) -> Rect {
        self.state.size
    }
    
    fn set_size(&mut self, size: Rect) {
        self.state.size = size;
    }
    
    fn has_focus(&self) -> bool {
        self.state.has_focus
    }
    
    fn set_focus(&mut self, focus: bool) {
        self.state.has_focus = focus;
    }
    
    fn is_visible(&self) -> bool {
        self.state.is_visible
    }
    
    fn set_visible(&mut self, visible: bool) {
        self.state.is_visible = visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::themes::Theme;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_splash_component_creation() {
        let splash = SplashComponent::new("v1.0.0".to_string());
        assert_eq!(splash.version, "v1.0.0");
        assert!(splash.show_info);
        assert!(!splash.compact_mode);
    }

    #[test]
    fn test_splash_component_with_options() {
        let splash = SplashComponent::new("v1.0.0".to_string())
            .with_info(false)
            .with_compact_mode(true);
        
        assert!(!splash.show_info);
        assert!(splash.compact_mode);
    }

    #[tokio::test]
    async fn test_splash_component_events() {
        let mut splash = SplashComponent::new("v1.0.0".to_string());
        
        // Test key event handling
        let key_event = KeyEvent::from(crossterm::event::KeyCode::Enter);
        assert!(splash.handle_key_event(key_event).await.is_ok());
        
        // Test mouse event handling
        let mouse_event = MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 0,
            row: 0,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        assert!(splash.handle_mouse_event(mouse_event).await.is_ok());
        
        // Test tick
        assert!(splash.tick().await.is_ok());
    }
}