//! Command palette dialog
//! 
//! This dialog provides a searchable command palette for quick access
//! to application functions and actions.

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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// A command that can be executed from the command palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub title: String,
    pub description: String,
    pub shortcut: Option<String>,
    pub category: String,
    pub enabled: bool,
}

impl Command {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            shortcut: None,
            category: category.into(),
            enabled: true,
        }
    }
    
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Commands dialog for executing application commands
pub struct CommandsDialog {
    /// Component state
    state: ComponentState,
    
    /// Dialog configuration
    config: DialogConfig,
    
    /// List of available commands
    commands: Vec<Command>,
    
    /// List state for navigation
    list_state: ListState,
    
    /// Event sender for dialog events
    event_sender: Option<mpsc::UnboundedSender<Event>>,
    
    /// Search/filter text
    filter_text: String,
    
    /// Current session ID (for context-sensitive commands)
    session_id: Option<String>,
    
    /// Loading state
    is_loading: bool,
    
    /// Error message if any
    error_message: Option<String>,
}

impl CommandsDialog {
    /// Create a new commands dialog
    pub fn new() -> Self {
        let config = DialogConfig::new(dialog_ids::commands())
            .with_title("Commands".to_string())
            .with_position(DialogPosition::Center)
            .with_size(DialogSize::Fixed(70, 20))
            .with_border(true)
            .modal(true)
            .closable(true);
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            state: ComponentState::new(),
            config,
            commands: Vec::new(),
            list_state,
            event_sender: None,
            filter_text: String::new(),
            session_id: None,
            is_loading: false,
            error_message: None,
        }
    }
    
    /// Set the event sender for this dialog
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<Event>) {
        self.event_sender = Some(sender);
    }
    
    /// Set the current session ID for context-sensitive commands
    pub fn set_session_id(&mut self, session_id: Option<String>) {
        self.session_id = session_id;
    }
    
    /// Load available commands
    pub async fn load_commands(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;
        
        let mut commands = vec![
            // Session management
            Command::new(
                "new_session",
                "New Session",
                "Create a new conversation session",
                "Session",
            ).with_shortcut("Ctrl+N"),
            
            Command::new(
                "switch_session",
                "Switch Session",
                "Switch to a different session",
                "Session",
            ).with_shortcut("Ctrl+S"),
            
            Command::new(
                "delete_session",
                "Delete Session",
                "Delete the current session",
                "Session",
            ).enabled(self.session_id.is_some()),
            
            // Model management
            Command::new(
                "switch_model",
                "Switch Model",
                "Change the AI model",
                "Model",
            ),
            
            // Application
            Command::new(
                "toggle_help",
                "Toggle Help",
                "Show or hide help information",
                "Application",
            ).with_shortcut("F1"),
            
            Command::new(
                "settings",
                "Settings",
                "Open application settings",
                "Application",
            ).with_shortcut("Ctrl+,"),
            
            Command::new(
                "quit",
                "Quit",
                "Exit the application",
                "Application",
            ).with_shortcut("Ctrl+Q"),
            
            // File operations
            Command::new(
                "open_file",
                "Open File",
                "Open a file in the conversation",
                "File",
            ).with_shortcut("Ctrl+O"),
            
            Command::new(
                "save_conversation",
                "Save Conversation",
                "Save the current conversation to a file",
                "File",
            ).enabled(self.session_id.is_some()),
            
            // Advanced
            Command::new(
                "clear_history",
                "Clear History",
                "Clear conversation history",
                "Advanced",
            ).enabled(self.session_id.is_some()),
            
            Command::new(
                "export_session",
                "Export Session",
                "Export session data",
                "Advanced",
            ).enabled(self.session_id.is_some()),
        ];
        
        // Add session-specific commands if we have a session
        if self.session_id.is_some() {
            commands.push(
                Command::new(
                    "summarize_session",
                    "Summarize Session",
                    "Create a summary of the current session",
                    "Session",
                )
            );
        }
        
        self.commands = commands;
        
        // Select first item if available
        if !self.commands.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        
        self.is_loading = false;
        Ok(())
    }
    
    /// Execute the selected command
    pub async fn execute_selected(&self) -> Result<()> {
        if let Some(index) = self.list_state.selected() {
            if let Some(command) = self.filtered_commands().get(index) {
                if !command.enabled {
                    return Ok(());
                }
                
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(Event::Custom(
                        "command_executed".to_string(),
                        serde_json::json!({
                            "command_id": command.id,
                            "command_title": command.title
                        }),
                    ));
                }
                self.close_dialog().await?;
            }
        }
        Ok(())
    }
    
    /// Get filtered commands based on search text
    fn filtered_commands(&self) -> Vec<&Command> {
        if self.filter_text.is_empty() {
            self.commands.iter().collect()
        } else {
            self.commands
                .iter()
                .filter(|command| {
                    command.title.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || command.description.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || command.category.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || command.id.to_lowercase().contains(&self.filter_text.to_lowercase())
                })
                .collect()
        }
    }
    
    /// Move selection up
    fn move_selection_up(&mut self) {
        let filtered_count = self.filtered_commands().len();
        if filtered_count == 0 {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = if current == 0 {
            filtered_count - 1
        } else {
            current - 1
        };
        self.list_state.select(Some(new_index));
    }
    
    /// Move selection down
    fn move_selection_down(&mut self) {
        let filtered_count = self.filtered_commands().len();
        if filtered_count == 0 {
            return;
        }
        
        let current = self.list_state.selected().unwrap_or(0);
        let new_index = if current + 1 >= filtered_count {
            0
        } else {
            current + 1
        };
        self.list_state.select(Some(new_index));
    }
    
    /// Close the dialog
    async fn close_dialog(&self) -> Result<()> {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(Event::Custom(
                "dialog_close_request".to_string(),
                serde_json::json!({"dialog_id": self.config.id.as_str()}),
            ));
        }
        Ok(())
    }
    
    /// Render the command list
    fn render_command_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let filtered_commands = self.filtered_commands();
        
        if self.is_loading {
            let loading = Paragraph::new("Loading commands...")
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
        
        if filtered_commands.is_empty() {
            let empty_msg = if self.filter_text.is_empty() {
                "No commands available."
            } else {
                "No commands match your search."
            };
            
            let empty = Paragraph::new(empty_msg)
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }
        
        // Group commands by category
        let mut categorized: HashMap<String, Vec<&Command>> = HashMap::new();
        for command in filtered_commands {
            categorized
                .entry(command.category.clone())
                .or_insert_with(Vec::new)
                .push(command);
        }
        
        let mut items = Vec::new();
        let mut item_index = 0;
        
        for (category, commands) in categorized.iter() {
            // Add category header
            items.push(ListItem::new(format!("── {} ──", category))
                .style(Style::default().fg(theme.text_muted()).add_modifier(Modifier::BOLD)));
            
            // Add commands in this category
            for command in commands {
                let mut line = command.title.clone();
                
                // Add shortcut if available
                if let Some(shortcut) = &command.shortcut {
                    line = format!("{} ({})", line, shortcut);
                }
                
                // Add description
                line = format!("{}\n    {}", line, command.description);
                
                let style = if command.enabled {
                    Style::default().fg(theme.text)
                } else {
                    Style::default().fg(theme.text_muted())
                };
                
                items.push(ListItem::new(line).style(style));
                item_index += 1;
            }
        }
        
        let list = List::new(items)
            .block(Block::default())
            .style(Style::default().fg(theme.text))
            .highlight_style(
                Style::default()
                    .bg(theme.primary)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            )
            .highlight_symbol("► ");
        
        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
    
    /// Render the search input
    fn render_search_input(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let search_text = if self.filter_text.is_empty() {
            "Type to search commands..."
        } else {
            &self.filter_text
        };
        
        let search_input = Paragraph::new(search_text)
            .style(Style::default().bg(theme.surface()).fg(theme.text))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        
        frame.render_widget(search_input, area);
    }
    
    /// Render help text
    fn render_help(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_text = "↑/↓: Navigate • Enter: Execute • Type: Search • Esc: Close";
        
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.text_muted()).add_modifier(Modifier::DIM))
            .alignment(Alignment::Center);
        
        frame.render_widget(help, area);
    }
}

#[async_trait]
impl Component for CommandsDialog {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match (event.code, event.modifiers) {
            // Navigation
            (KeyCode::Up | KeyCode::Char('k'), _) => {
                self.move_selection_up();
            }
            (KeyCode::Down | KeyCode::Char('j'), _) => {
                self.move_selection_down();
            }
            
            // Execute command
            (KeyCode::Enter, _) => {
                self.execute_selected().await?;
            }
            
            // Close
            (KeyCode::Esc, _) => {
                self.close_dialog().await?;
            }
            
            // Search/filter
            (KeyCode::Backspace, _) => {
                self.filter_text.pop();
                // Reset selection when filter changes
                if !self.filtered_commands().is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            
            (KeyCode::Char(c), _) => {
                self.filter_text.push(c);
                // Reset selection when filter changes
                if !self.filtered_commands().is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            
            _ => {}
        }
        
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        // TODO: Implement mouse handling for list selection
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
impl Dialog for CommandsDialog {
    fn config(&self) -> &DialogConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut DialogConfig {
        &mut self.config
    }
    
    fn position(&self, available_area: Rect) -> (u16, u16) {
        let (width, height) = self.dialog_size(available_area);
        let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
        let y = available_area.y + (available_area.height.saturating_sub(height)) / 4; // Positioned higher
        (x, y)
    }
    
    fn dialog_size(&self, _available_area: Rect) -> (u16, u16) {
        (70, 20)
    }
    
    async fn on_open(&mut self) -> Result<()> {
        self.load_commands().await?;
        Ok(())
    }
    
    fn render_content(&mut self, frame: &mut Frame, content_area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Search input
                Constraint::Min(5),      // Command list
                Constraint::Length(1),   // Help text
            ])
            .split(content_area);
        
        // Render search input
        self.render_search_input(frame, chunks[0], theme);
        
        // Render command list
        self.render_command_list(frame, chunks[1], theme);
        
        // Render help
        self.render_help(frame, chunks[2], theme);
    }
    
    fn min_size(&self) -> (u16, u16) {
        (50, 15)
    }
    
    fn preferred_size(&self) -> (u16, u16) {
        (70, 20)
    }
}

impl Default for CommandsDialog {
    fn default() -> Self {
        Self::new()
    }
}