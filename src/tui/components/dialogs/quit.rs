//! Quit confirmation dialog
//! 
//! This dialog asks the user to confirm before quitting the application.
//! It provides "Yes" and "No" options with keyboard navigation.

use super::types::{Dialog, DialogConfig, DialogId, DialogPosition, DialogSize, dialog_ids};
use crate::tui::{
    components::{Component, ComponentState},
    events::Event,
    themes::Theme,
    Frame,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, Paragraph},
};
use tokio::sync::mpsc;

/// Quit dialog for confirming application exit
pub struct QuitDialog {
    /// Component state
    state: ComponentState,
    
    /// Dialog configuration
    config: DialogConfig,
    
    /// Currently selected option (true = Yes, false = No)
    selected_yes: bool,
    
    /// Event sender for dialog events
    event_sender: Option<mpsc::UnboundedSender<Event>>,
    
    /// Question text to display
    question: String,
    
    /// Button labels
    yes_label: String,
    no_label: String,
}

impl QuitDialog {
    /// Create a new quit dialog
    pub fn new() -> Self {
        let config = DialogConfig::new(dialog_ids::quit())
            .with_title("Confirm Quit".to_string())
            .with_position(DialogPosition::Center)
            .with_size(DialogSize::Fixed(40, 7))
            .with_border(true)
            .modal(true)
            .closable(true);
        
        Self {
            state: ComponentState::new(),
            config,
            selected_yes: false, // Default to "No" for safety
            event_sender: None,
            question: "Are you sure you want to quit?".to_string(),
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
        }
    }
    
    /// Create a quit dialog with custom question
    pub fn with_question(question: impl Into<String>) -> Self {
        let mut dialog = Self::new();
        dialog.question = question.into();
        dialog
    }
    
    /// Set the event sender for this dialog
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<Event>) {
        self.event_sender = Some(sender);
    }
    
    /// Set custom button labels
    pub fn set_button_labels(&mut self, yes_label: impl Into<String>, no_label: impl Into<String>) {
        self.yes_label = yes_label.into();
        self.no_label = no_label.into();
    }
    
    /// Get the currently selected option
    pub fn selected_yes(&self) -> bool {
        self.selected_yes
    }
    
    /// Set the selected option
    pub fn set_selected_yes(&mut self, selected: bool) {
        self.selected_yes = selected;
    }
    
    /// Toggle selection between Yes and No
    fn toggle_selection(&mut self) {
        self.selected_yes = !self.selected_yes;
    }
    
    /// Handle confirmation (user pressed Enter or clicked a button)
    async fn handle_confirm(&self) -> Result<()> {
        if let Some(sender) = &self.event_sender {
            if self.selected_yes {
                // User confirmed quit - send quit event
                let _ = sender.send(Event::Custom(
                    "quit_confirmed".to_string(),
                    serde_json::json!({"confirmed": true}),
                ));
            } else {
                // User cancelled - close dialog
                let _ = sender.send(Event::Custom(
                    "dialog_close_request".to_string(),
                    serde_json::json!({"dialog_id": self.config.id.as_str()}),
                ));
            }
        }
        Ok(())
    }
    
    /// Handle cancellation (user pressed Escape or clicked No)
    async fn handle_cancel(&self) -> Result<()> {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(Event::Custom(
                "dialog_close_request".to_string(),
                serde_json::json!({"dialog_id": self.config.id.as_str()}),
            ));
        }
        Ok(())
    }
    
    /// Render the dialog buttons
    fn render_buttons(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Create button layout
        let button_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(area);
        
        // Style buttons based on selection
        let yes_style = if self.selected_yes {
            Style::default()
                .bg(theme.primary)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(theme.surface())
                .fg(theme.text)
        };
        
        let no_style = if !self.selected_yes {
            Style::default()
                .bg(theme.primary)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .bg(theme.surface())
                .fg(theme.text)
        };
        
        // Render Yes button
        let yes_button = Paragraph::new(format!(" {} ", self.yes_label))
            .style(yes_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(yes_button, button_layout[0]);
        
        // Render No button
        let no_button = Paragraph::new(format!(" {} ", self.no_label))
            .style(no_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        
        frame.render_widget(no_button, button_layout[1]);
    }
}

#[async_trait]
impl Component for QuitDialog {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match (event.code, event.modifiers) {
            // Navigation between buttons
            (KeyCode::Left | KeyCode::Right | KeyCode::Tab, _) => {
                self.toggle_selection();
            }
            
            // Confirm selection
            (KeyCode::Enter | KeyCode::Char(' '), _) => {
                self.handle_confirm().await?;
            }
            
            // Direct Yes/No shortcuts
            (KeyCode::Char('y') | KeyCode::Char('Y'), _) => {
                self.selected_yes = true;
                self.handle_confirm().await?;
            }
            
            (KeyCode::Char('n') | KeyCode::Char('N'), _) => {
                self.selected_yes = false;
                self.handle_confirm().await?;
            }
            
            // Cancel (Escape)
            (KeyCode::Esc, _) => {
                self.handle_cancel().await?;
            }
            
            // Quit directly with Ctrl+C
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.selected_yes = true;
                self.handle_confirm().await?;
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        // TODO: Implement mouse handling for button clicks
        let _ = event;
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        self.render_content(frame, area, theme);
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

#[async_trait]
impl Dialog for QuitDialog {
    fn config(&self) -> &DialogConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut DialogConfig {
        &mut self.config
    }
    
    fn position(&self, available_area: Rect) -> (u16, u16) {
        // Center the dialog
        let width = 40;
        let height = 7;
        let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
        let y = available_area.y + (available_area.height.saturating_sub(height)) / 2;
        (x, y)
    }
    
    fn dialog_size(&self, _available_area: Rect) -> (u16, u16) {
        (40, 7)
    }
    
    async fn handle_dialog_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Handle Escape key for closing
        if key.code == KeyCode::Esc && key.modifiers.is_empty() {
            self.handle_cancel().await?;
            return Ok(true); // Event handled
        }
        
        Ok(false) // Let normal key handling proceed
    }
    
    fn render_content(&mut self, frame: &mut Frame, content_area: Rect, theme: &Theme) {
        // Create layout for question and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),     // Question area
                Constraint::Length(3),  // Button area
            ])
            .split(content_area);
        
        // Render question
        let question_paragraph = Paragraph::new(self.question.clone())
            .style(Style::default().fg(theme.text))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        
        frame.render_widget(question_paragraph, chunks[0]);
        
        // Render buttons
        self.render_buttons(frame, chunks[1], theme);
        
        // Add help text at the bottom
        if chunks.len() > 1 && chunks[1].height > 3 {
            let help_area = Rect {
                x: chunks[1].x,
                y: chunks[1].y + 3,
                width: chunks[1].width,
                height: 1,
            };
            
            let help_text = "↑/↓/Tab: Select • Enter/Space: Confirm • Y/N: Direct • Esc: Cancel";
            let help_paragraph = Paragraph::new(help_text)
                .style(Style::default().fg(theme.text_muted()).add_modifier(Modifier::DIM))
                .alignment(Alignment::Center);
            
            frame.render_widget(help_paragraph, help_area);
        }
    }
    
    fn min_size(&self) -> (u16, u16) {
        (30, 5)
    }
    
    fn preferred_size(&self) -> (u16, u16) {
        (40, 7)
    }
    
    fn max_size(&self) -> Option<(u16, u16)> {
        Some((60, 10))
    }
}

impl Default for QuitDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a quit dialog with event sender
pub fn create_quit_dialog(event_sender: mpsc::UnboundedSender<Event>) -> QuitDialog {
    let mut dialog = QuitDialog::new();
    dialog.set_event_sender(event_sender);
    dialog
}