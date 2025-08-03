//! Session management dialog
//! 
//! This dialog allows users to view, create, switch between, and manage
//! conversation sessions.

use super::types::{Dialog, DialogConfig, DialogId, DialogPosition, DialogSize, dialog_ids};
use crate::{
    session::{Session, SessionManager},
    tui::{
        components::{Component, ComponentState},
        events::Event,
        themes::Theme,
        Frame,
    },
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

/// Sessions dialog for managing conversation sessions
pub struct SessionsDialog {
    /// Component state
    state: ComponentState,
    
    /// Dialog configuration
    config: DialogConfig,
    
    /// List of sessions
    sessions: Vec<Session>,
    
    /// List state for navigation
    list_state: ListState,
    
    /// Event sender for dialog events
    event_sender: Option<mpsc::UnboundedSender<Event>>,
    
    /// Session manager for loading/creating sessions (removed for now due to Send/Sync issues)
    // session_manager: Option<Arc<RwLock<SessionManager>>>,
    
    /// Filter text for searching sessions
    filter_text: String,
    
    /// Whether we're in filter/search mode
    in_search_mode: bool,
    
    /// Loading state
    is_loading: bool,
    
    /// Error message if any
    error_message: Option<String>,
}

impl SessionsDialog {
    /// Create a new sessions dialog
    pub fn new() -> Self {
        let config = DialogConfig::new(dialog_ids::sessions())
            .with_title("Sessions".to_string())
            .with_position(DialogPosition::Center)
            .with_size(DialogSize::Percentage(70, 80))
            .with_border(true)
            .modal(true)
            .closable(true);
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            state: ComponentState::new(),
            config,
            sessions: Vec::new(),
            list_state,
            event_sender: None,
            // session_manager: None,
            filter_text: String::new(),
            in_search_mode: false,
            is_loading: false,
            error_message: None,
        }
    }
    
    /// Set the event sender for this dialog
    pub fn set_event_sender(&mut self, sender: mpsc::UnboundedSender<Event>) {
        self.event_sender = Some(sender);
    }
    
    /// Set the session manager (disabled for now)
    // pub fn set_session_manager(&mut self, manager: Arc<RwLock<SessionManager>>) {
    //     self.session_manager = Some(manager);
    // }
    
    /// Load sessions from the session manager (mock implementation for now)
    pub async fn load_sessions(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;
        
        // Mock implementation - create some fake sessions for demo
        use chrono::Utc;
        self.sessions = vec![
            Session {
                id: "session1".to_string(),
                title: "Example Session 1".to_string(),
                parent_session_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                message_count: 5,
                token_usage: Default::default(),
                total_cost: 0.0,
                metadata: Default::default(),
            },
            Session {
                id: "session2".to_string(),
                title: "Example Session 2".to_string(),
                parent_session_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                message_count: 12,
                token_usage: Default::default(),
                total_cost: 0.0,
                metadata: Default::default(),
            },
        ];
        
        if !self.sessions.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
        
        self.is_loading = false;
        Ok(())
    }
    
    /// Create a new session (mock implementation)
    pub async fn create_new_session(&mut self) -> Result<()> {
        let title = format!("Session {}", chrono::Utc::now().format("%Y-%m-%d %H:%M"));
        let new_session_id = format!("session_{}", Uuid::new_v4());
        
        // Send event to switch to the new session
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(Event::Custom(
                "session_selected".to_string(),
                serde_json::json!({"session_id": new_session_id}),
            ));
        }
        self.close_dialog().await?;
        
        Ok(())
    }
    
    /// Switch to the selected session
    pub async fn switch_to_selected(&self) -> Result<()> {
        if let Some(index) = self.list_state.selected() {
            if let Some(session) = self.filtered_sessions().get(index) {
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(Event::Custom(
                        "session_selected".to_string(),
                        serde_json::json!({"session_id": session.id}),
                    ));
                }
                self.close_dialog().await?;
            }
        }
        Ok(())
    }
    
    /// Delete the selected session
    pub async fn delete_selected(&mut self) -> Result<()> {
        let session_id = if let Some(index) = self.list_state.selected() {
            if let Some(session) = self.filtered_sessions().get(index) {
                Some(session.id.clone())
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(_session_id) = session_id {
            // Mock deletion - just reload sessions
            self.load_sessions().await?;
        }
        Ok(())
    }
    
    /// Get filtered sessions based on search text
    fn filtered_sessions(&self) -> Vec<&Session> {
        if self.filter_text.is_empty() {
            self.sessions.iter().collect()
        } else {
            self.sessions
                .iter()
                .filter(|session| {
                    session.title.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || session.id.contains(&self.filter_text)
                })
                .collect()
        }
    }
    
    /// Move selection up
    fn move_selection_up(&mut self) {
        let filtered_count = self.filtered_sessions().len();
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
        let filtered_count = self.filtered_sessions().len();
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
    
    /// Render the session list
    fn render_session_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let filtered_sessions = self.filtered_sessions();
        
        if self.is_loading {
            let loading = Paragraph::new("Loading sessions...")
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
        
        if filtered_sessions.is_empty() {
            let empty_msg = if self.filter_text.is_empty() {
                "No sessions found. Press 'n' to create a new session."
            } else {
                "No sessions match your search. Press 'n' to create a new session."
            };
            
            let empty = Paragraph::new(empty_msg)
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }
        
        let items: Vec<ListItem> = filtered_sessions
            .iter()
            .map(|session| {
                let date = session.created_at.format("%Y-%m-%d %H:%M").to_string();
                let line = format!("{} - {} ({} messages)", 
                    session.title, 
                    date, 
                    session.message_count
                );
                ListItem::new(line)
            })
            .collect();
        
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
    
    /// Render the search bar
    fn render_search_bar(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let search_style = if self.in_search_mode {
            Style::default().bg(theme.primary).fg(Color::White)
        } else {
            Style::default().bg(theme.surface()).fg(theme.text)
        };
        
        let search_text = if self.filter_text.is_empty() && !self.in_search_mode {
            "Press '/' to search sessions..."
        } else {
            &self.filter_text
        };
        
        let search_bar = Paragraph::new(search_text)
            .style(search_style)
            .block(Block::default().borders(Borders::ALL).title("Search"));
        
        frame.render_widget(search_bar, area);
    }
    
    /// Render help text
    fn render_help(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let help_text = if self.in_search_mode {
            "Enter: Confirm search • Esc: Exit search • Backspace: Delete"
        } else {
            "↑/↓: Navigate • Enter: Select • n: New • d: Delete • /: Search • Esc: Close"
        };
        
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.text_muted()).add_modifier(Modifier::DIM))
            .alignment(Alignment::Center);
        
        frame.render_widget(help, area);
    }
}

#[async_trait]
impl Component for SessionsDialog {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        if self.in_search_mode {
            match event.code {
                KeyCode::Esc => {
                    self.in_search_mode = false;
                }
                KeyCode::Enter => {
                    self.in_search_mode = false;
                    // Apply filter
                    if self.list_state.selected().is_none() && !self.filtered_sessions().is_empty() {
                        self.list_state.select(Some(0));
                    }
                }
                KeyCode::Backspace => {
                    self.filter_text.pop();
                }
                KeyCode::Char(c) => {
                    self.filter_text.push(c);
                }
                _ => {}
            }
        } else {
            match (event.code, event.modifiers) {
                // Navigation
                (KeyCode::Up | KeyCode::Char('k'), _) => {
                    self.move_selection_up();
                }
                (KeyCode::Down | KeyCode::Char('j'), _) => {
                    self.move_selection_down();
                }
                
                // Selection
                (KeyCode::Enter, _) => {
                    self.switch_to_selected().await?;
                }
                
                // New session
                (KeyCode::Char('n') | KeyCode::Char('N'), _) => {
                    self.create_new_session().await?;
                }
                
                // Delete session
                (KeyCode::Char('d') | KeyCode::Char('D'), _) => {
                    self.delete_selected().await?;
                }
                
                // Search
                (KeyCode::Char('/'), _) => {
                    self.in_search_mode = true;
                    self.filter_text.clear();
                }
                
                // Refresh
                (KeyCode::Char('r') | KeyCode::Char('R'), _) => {
                    self.load_sessions().await?;
                }
                
                // Close
                (KeyCode::Esc | KeyCode::Char('q'), _) => {
                    self.close_dialog().await?;
                }
                
                _ => {}
            }
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
impl Dialog for SessionsDialog {
    fn config(&self) -> &DialogConfig {
        &self.config
    }
    
    fn config_mut(&mut self) -> &mut DialogConfig {
        &mut self.config
    }
    
    fn position(&self, available_area: Rect) -> (u16, u16) {
        let (width, height) = self.dialog_size(available_area);
        let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
        let y = available_area.y + (available_area.height.saturating_sub(height)) / 2;
        (x, y)
    }
    
    fn dialog_size(&self, available_area: Rect) -> (u16, u16) {
        let width = (available_area.width as f32 * 0.7) as u16;
        let height = (available_area.height as f32 * 0.8) as u16;
        (width.max(50), height.max(15))
    }
    
    async fn on_open(&mut self) -> Result<()> {
        self.load_sessions().await?;
        Ok(())
    }
    
    fn render_content(&mut self, frame: &mut Frame, content_area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Search bar
                Constraint::Min(5),      // Session list
                Constraint::Length(1),   // Help text
            ])
            .split(content_area);
        
        // Render search bar
        self.render_search_bar(frame, chunks[0], theme);
        
        // Render session list
        self.render_session_list(frame, chunks[1], theme);
        
        // Render help
        self.render_help(frame, chunks[2], theme);
        
        // Render error message if any
        if let Some(error) = &self.error_message {
            let error_area = Rect {
                x: chunks[1].x,
                y: chunks[1].y + chunks[1].height - 3,
                width: chunks[1].width,
                height: 3,
            };
            
            let error_paragraph = Paragraph::new(error.clone())
                .style(Style::default().fg(Color::Red))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Error"));
            
            frame.render_widget(Clear, error_area);
            frame.render_widget(error_paragraph, error_area);
        }
    }
    
    fn min_size(&self) -> (u16, u16) {
        (40, 15)
    }
    
    fn preferred_size(&self) -> (u16, u16) {
        (60, 25)
    }
}

impl Default for SessionsDialog {
    fn default() -> Self {
        Self::new()
    }
}