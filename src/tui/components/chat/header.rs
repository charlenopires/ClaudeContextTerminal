//! Chat header component for displaying session information
//!
//! This module provides a header component that shows session details,
//! model information, token usage, and various status indicators.

use super::message_types::ChatMessage;
use crate::{
    session::{Session}, // Conversation temporarily disabled due to Send/Sync issues
    tui::{
        components::{Component, ComponentState},
        themes::{Theme, ThemeManager},
        Frame,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Gauge, Paragraph, Wrap},
};
use std::time::{Duration, Instant};

/// Chat header component
pub struct ChatHeader {
    state: ComponentState,
    theme_manager: ThemeManager,
    session: Option<Session>,
    // TODO: Re-enable when Conversation is Send+Sync
    // conversation: Option<Conversation>,
    
    // Display options
    show_details: bool,
    show_model_info: bool,
    show_token_usage: bool,
    show_session_stats: bool,
    compact_mode: bool,
    
    // Animation state
    last_update: Instant,
    blink_state: bool,
    
    // Cached information
    cached_session_title: String,
    cached_model_name: String,
    cached_provider_name: String,
    cached_token_count: u64,
    cached_context_window: u64,
    cached_cost: f64,
}

/// Header section configuration
#[derive(Debug, Clone)]
pub struct HeaderConfig {
    pub show_logo: bool,
    pub show_session_info: bool,
    pub show_model_info: bool,
    pub show_token_usage: bool,
    pub show_working_directory: bool,
    pub show_git_info: bool,
    pub show_time: bool,
    pub compact_mode: bool,
    pub auto_hide_when_inactive: bool,
    pub max_title_length: usize,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            show_logo: true,
            show_session_info: true,
            show_model_info: true,
            show_token_usage: true,
            show_working_directory: false,
            show_git_info: false,
            show_time: false,
            compact_mode: false,
            auto_hide_when_inactive: false,
            max_title_length: 50,
        }
    }
}

impl ChatHeader {
    /// Create a new chat header
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(),
            theme_manager: ThemeManager::new(),
            session: None,
            // conversation: None,
            show_details: false,
            show_model_info: true,
            show_token_usage: true,
            show_session_stats: true,
            compact_mode: false,
            last_update: Instant::now(),
            blink_state: false,
            cached_session_title: String::new(),
            cached_model_name: String::new(),
            cached_provider_name: String::new(),
            cached_token_count: 0,
            cached_context_window: 0,
            cached_cost: 0.0,
        }
    }

    /// Create header with configuration
    pub fn with_config(config: HeaderConfig) -> Self {
        let mut header = Self::new();
        header.show_model_info = config.show_model_info;
        header.show_token_usage = config.show_token_usage;
        header.show_session_stats = config.show_session_info;
        header.compact_mode = config.compact_mode;
        header
    }

    /// Set the current session
    pub fn set_session(&mut self, session: Option<Session>) {
        self.session = session;
        self.update_cached_info();
    }

    /// Set the current conversation
    // TODO: Re-enable when Conversation is Send+Sync
    // pub fn set_conversation(&mut self, conversation: Option<Conversation>) {
    //     self.conversation = conversation;
    //     self.update_cached_info();
    // }

    /// Toggle details view
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Set details view
    pub fn set_show_details(&mut self, show: bool) {
        self.show_details = show;
    }

    /// Get details view state
    pub fn is_showing_details(&self) -> bool {
        self.show_details
    }

    /// Set compact mode
    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }

    /// Update cached information from session and conversation
    fn update_cached_info(&mut self) {
        if let Some(ref session) = self.session {
            self.cached_session_title = session.title.clone();
            
            // Update model and provider info
            // In a real implementation, this would come from the session's model configuration
            self.cached_model_name = "gpt-4".to_string(); // Placeholder
            self.cached_provider_name = "OpenAI".to_string(); // Placeholder
            self.cached_context_window = 128000; // Placeholder
        }
        
        // TODO: Re-enable when Conversation is Send+Sync
        // if let Some(ref conversation) = self.conversation {
        //     // Calculate token usage from conversation messages
        //     self.cached_token_count = self.calculate_token_usage(&conversation);
        //     self.cached_cost = self.calculate_cost(&conversation);
        // }
    }

    /// Calculate approximate token usage
    // TODO: Re-enable when Conversation is Send+Sync
    // fn calculate_token_usage(&self, _conversation: &Conversation) -> u64 {
    //     // This is a simplified calculation
    //     // In a real implementation, you'd use the actual token counts from the LLM responses
    //     0 // Placeholder
    // }

    /// Calculate approximate cost
    // TODO: Re-enable when Conversation is Send+Sync
    // fn calculate_cost(&self, _conversation: &Conversation) -> f64 {
    //     // This would calculate based on actual token usage and model pricing
    //     0.0 // Placeholder
    // }

    /// Render the header in normal mode
    fn render_normal_mode(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        // Split into left, center, and right sections
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20),  // Logo/brand
                Constraint::Min(1),      // Session info
                Constraint::Length(30),  // Model/stats
            ])
            .split(area);

        // Render logo/brand
        self.render_logo_section(frame, chunks[0]);
        
        // Render session info
        self.render_session_section(frame, chunks[1]);
        
        // Render model/stats section
        self.render_stats_section(frame, chunks[2]);
    }

    /// Render the header in compact mode
    fn render_compact_mode(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        let mut spans = Vec::new();
        
        // Session title
        if let Some(ref session) = self.session {
            let title = if session.title.len() > 25 {
                format!("{}...", &session.title[..22])
            } else {
                session.title.clone()
            };
            
            spans.push(Span::styled(title, theme.styles.title));
            spans.push(Span::raw(" • "));
        }
        
        // Model info
        if self.show_model_info {
            spans.push(Span::styled(
                format!("{}", self.cached_model_name),
                theme.styles.info,
            ));
            spans.push(Span::raw(" • "));
        }
        
        // Token usage
        if self.show_token_usage && self.cached_context_window > 0 {
            let percentage = (self.cached_token_count as f64 / self.cached_context_window as f64) * 100.0;
            let style = if percentage > 80.0 {
                theme.styles.warning
            } else if percentage > 60.0 {
                theme.styles.info
            } else {
                theme.styles.muted
            };
            
            spans.push(Span::styled(
                format!("{:.0}%", percentage),
                style,
            ));
        }
        
        let header_line = Line::from(spans);
        let paragraph = Paragraph::new(header_line)
            .style(theme.styles.base)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }

    /// Render logo/brand section
    fn render_logo_section(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        let logo_text = if area.width > 15 {
            "🤖 Goofy"
        } else {
            "🤖"
        };
        
        let logo = Paragraph::new(logo_text)
            .style(theme.styles.title.add_modifier(Modifier::BOLD))
            .wrap(Wrap { trim: true });
        
        frame.render_widget(logo, area);
    }

    /// Render session information section
    fn render_session_section(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        if let Some(ref session) = self.session {
            let mut lines = Vec::new();
            
            // Session title
            let title = if session.title.len() > area.width as usize - 4 {
                format!("{}...", &session.title[..area.width as usize - 7])
            } else {
                session.title.clone()
            };
            
            lines.push(Line::from(vec![
                Span::styled("📝 ", theme.styles.info),
                Span::styled(title, theme.styles.text),
            ]));
            
            // Additional info if showing details
            if self.show_details {
                lines.push(Line::from(vec![
                    Span::styled("ID: ", theme.styles.muted),
                    Span::styled(&session.id[..8], theme.styles.muted),
                ]));
                
                let created_at = session.created_at.format("%Y-%m-%d %H:%M").to_string();
                lines.push(Line::from(vec![
                    Span::styled("Created: ", theme.styles.muted),
                    Span::styled(created_at, theme.styles.muted),
                ]));
            }
            
            let session_info = Paragraph::new(Text::from(lines))
                .style(theme.styles.base)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(session_info, area);
        } else {
            // No session selected
            let no_session = Paragraph::new("No session selected")
                .style(theme.styles.muted)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(no_session, area);
        }
    }

    /// Render statistics section
    fn render_stats_section(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        let mut lines = Vec::new();
        
        // Model information
        if self.show_model_info {
            lines.push(Line::from(vec![
                Span::styled("🤖 ", theme.styles.info),
                Span::styled(&self.cached_model_name, theme.styles.text),
                if !self.cached_provider_name.is_empty() {
                    Span::styled(format!(" ({})", self.cached_provider_name), theme.styles.muted)
                } else {
                    Span::raw("")
                },
            ]));
        }
        
        // Token usage
        if self.show_token_usage && self.cached_context_window > 0 {
            let percentage = (self.cached_token_count as f64 / self.cached_context_window as f64) * 100.0;
            let style = if percentage > 80.0 {
                theme.styles.warning
            } else if percentage > 60.0 {
                theme.styles.info
            } else {
                theme.styles.success
            };
            
            lines.push(Line::from(vec![
                Span::styled("📊 ", theme.styles.info),
                Span::styled(
                    format!("{:.0}% ", percentage),
                    style,
                ),
                Span::styled(
                    format!("({}/{})", 
                        format_number(self.cached_token_count),
                        format_number(self.cached_context_window)
                    ),
                    theme.styles.muted,
                ),
            ]));
        }
        
        // Cost information
        if self.show_session_stats && self.cached_cost > 0.0 {
            lines.push(Line::from(vec![
                Span::styled("💰 ", theme.styles.info),
                Span::styled(
                    format!("${:.4}", self.cached_cost),
                    theme.styles.text,
                ),
            ]));
        }
        
        let stats_info = Paragraph::new(Text::from(lines))
            .style(theme.styles.base)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(stats_info, area);
    }

    /// Render detailed view
    fn render_detailed_view(&self, frame: &mut Frame, area: Rect) {
        let theme = self.theme_manager.current_theme();
        
        // Create a bordered block for the detailed view
        let block = Block::default()
            .title("Session Details")
            .borders(Borders::ALL)
            .border_style(theme.styles.dialog_border);
        
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        let mut lines = Vec::new();
        
        if let Some(ref session) = self.session {
            // Session information
            lines.push(Line::from(vec![
                Span::styled("Session: ", theme.styles.subtitle),
                Span::styled(&session.title, theme.styles.text),
            ]));
            
            lines.push(Line::from(vec![
                Span::styled("ID: ", theme.styles.muted),
                Span::styled(&session.id, theme.styles.muted),
            ]));
            
            lines.push(Line::from(vec![
                Span::styled("Created: ", theme.styles.muted),
                Span::styled(
                    session.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                    theme.styles.muted,
                ),
            ]));
            
            lines.push(Line::from(Span::raw(""))); // Empty line
            
            // Model information
            lines.push(Line::from(vec![
                Span::styled("Model: ", theme.styles.subtitle),
                Span::styled(&self.cached_model_name, theme.styles.text),
            ]));
            
            lines.push(Line::from(vec![
                Span::styled("Provider: ", theme.styles.muted),
                Span::styled(&self.cached_provider_name, theme.styles.text),
            ]));
            
            lines.push(Line::from(vec![
                Span::styled("Context Window: ", theme.styles.muted),
                Span::styled(
                    format!("{} tokens", format_number(self.cached_context_window)),
                    theme.styles.text,
                ),
            ]));
            
            lines.push(Line::from(Span::raw(""))); // Empty line
            
            // Usage statistics
            lines.push(Line::from(Span::styled("Usage Statistics", theme.styles.subtitle)));
            
            lines.push(Line::from(vec![
                Span::styled("Tokens Used: ", theme.styles.muted),
                Span::styled(
                    format!("{}", format_number(self.cached_token_count)),
                    theme.styles.text,
                ),
            ]));
            
            if self.cached_context_window > 0 {
                let percentage = (self.cached_token_count as f64 / self.cached_context_window as f64) * 100.0;
                lines.push(Line::from(vec![
                    Span::styled("Usage: ", theme.styles.muted),
                    Span::styled(
                        format!("{:.1}%", percentage),
                        if percentage > 80.0 { theme.styles.warning } else { theme.styles.text },
                    ),
                ]));
            }
            
            if self.cached_cost > 0.0 {
                lines.push(Line::from(vec![
                    Span::styled("Estimated Cost: ", theme.styles.muted),
                    Span::styled(
                        format!("${:.4}", self.cached_cost),
                        theme.styles.text,
                    ),
                ]));
            }
            
            // TODO: Re-enable when Conversation is Send+Sync
            // if let Some(ref conversation) = self.conversation {
            //     lines.push(Line::from(vec![
            //         Span::styled("Messages: ", theme.styles.muted),
            //         Span::styled(
            //             conversation.messages.len().to_string(),
            //             theme.styles.text,
            //         ),
            //     ]));
            // }
        }
        
        let details = Paragraph::new(Text::from(lines))
            .style(theme.styles.base)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(details, inner_area);
    }

    /// Calculate the minimum height needed for the header
    pub fn min_height(&self) -> u16 {
        if self.compact_mode {
            1
        } else if self.show_details {
            8 // Enough for detailed view
        } else {
            2 // Normal mode
        }
    }

    /// Calculate the preferred height for the header
    pub fn preferred_height(&self) -> u16 {
        if self.compact_mode {
            1
        } else if self.show_details {
            12 // More space for detailed view
        } else {
            3 // Normal mode with some padding
        }
    }
}

#[async_trait]
impl Component for ChatHeader {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;
        
        match event.code {
            KeyCode::F(1) => {
                self.toggle_details();
            }
            _ => {}
        }
        
        Ok(())
    }

    async fn handle_mouse_event(&mut self, _event: MouseEvent) -> Result<()> {
        // TODO: Handle mouse clicks for toggling details, etc.
        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Update blink state for animations
        if self.last_update.elapsed() >= Duration::from_millis(500) {
            self.blink_state = !self.blink_state;
            self.last_update = Instant::now();
        }
        
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if self.show_details {
            self.render_detailed_view(frame, area);
        } else if self.compact_mode {
            self.render_compact_mode(frame, area);
        } else {
            self.render_normal_mode(frame, area);
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

impl Default for ChatHeader {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn format_number(num: u64) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = ChatHeader::new();
        assert!(!header.show_details);
        assert!(header.show_model_info);
        assert!(!header.compact_mode);
    }

    #[test]
    fn test_details_toggle() {
        let mut header = ChatHeader::new();
        assert!(!header.is_showing_details());
        
        header.toggle_details();
        assert!(header.is_showing_details());
        
        header.toggle_details();
        assert!(!header.is_showing_details());
    }

    #[test]
    fn test_compact_mode() {
        let mut header = ChatHeader::new();
        assert_eq!(header.min_height(), 2);
        
        header.set_compact_mode(true);
        assert_eq!(header.min_height(), 1);
    }

    #[test]
    fn test_number_formatting() {
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1.0K");
        assert_eq!(format_number(1500), "1.5K");
        assert_eq!(format_number(1000000), "1.0M");
        assert_eq!(format_number(1500000), "1.5M");
    }
}