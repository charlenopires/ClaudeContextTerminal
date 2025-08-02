use super::{Component, ComponentState, ListView};
use crate::tui::{styles::Theme, Frame};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// Message role types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "User"),
            MessageRole::Assistant => write!(f, "Assistant"),
            MessageRole::System => write!(f, "System"),
            MessageRole::Tool => write!(f, "Tool"),
        }
    }
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            content,
            timestamp: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn user(content: String) -> Self {
        Self::new(MessageRole::User, content)
    }
    
    pub fn assistant(content: String) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
    
    pub fn system(content: String) -> Self {
        Self::new(MessageRole::System, content)
    }
    
    pub fn tool(content: String) -> Self {
        Self::new(MessageRole::Tool, content)
    }
}

/// Chat message list component
pub struct ChatMessageList {
    state: ComponentState,
    messages: VecDeque<ChatMessage>,
    list_state: ListState,
    max_messages: usize,
    auto_scroll: bool,
}

impl ChatMessageList {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(),
            messages: VecDeque::new(),
            list_state: ListState::default(),
            max_messages: 1000,
            auto_scroll: true,
        }
    }
    
    pub fn with_auto_scroll(mut self, auto_scroll: bool) -> Self {
        self.auto_scroll = auto_scroll;
        self
    }
    
    pub fn with_timestamps(self, _show_timestamps: bool) -> Self {
        // Implement timestamp display logic
        self
    }
}

#[async_trait]
impl Component for ChatMessageList {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Up => {
                // Move selection up
            }
            KeyCode::Down => {
                // Move selection down
            }
            _ => {}
        }
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Chat Messages")
            .style(theme.border_style());
        
        let items: Vec<ListItem> = self.messages
            .iter()
            .map(|msg| {
                let content = format!("{}: {}", msg.role, msg.content);
                ListItem::new(content).style(theme.text_style())
            })
            .collect();
        
        let list = List::new(items)
            .block(block)
            .highlight_style(theme.selection_style());
        
        frame.render_stateful_widget(list, area, &mut self.list_state);
        self.state.size = area;
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
impl ListView<ChatMessage> for ChatMessageList {
    async fn add_item(&mut self, message: ChatMessage) -> Result<()> {
        self.messages.push_back(message);
        
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
        
        Ok(())
    }
    
    async fn remove_item(&mut self, index: usize) -> Result<()> {
        if index < self.messages.len() {
            self.messages.remove(index);
        }
        Ok(())
    }
    
    async fn clear_items(&mut self) -> Result<()> {
        self.messages.clear();
        self.list_state.select(None);
        Ok(())
    }
    
    fn get_items(&self) -> &[ChatMessage] {
        // Convert VecDeque to slice - this is a limitation
        let (first, _) = self.messages.as_slices();
        first
    }
    
    fn selected_index(&self) -> Option<usize> {
        self.list_state.selected()
    }
    
    fn set_selected_index(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }
    
    async fn move_selection_up(&mut self) -> Result<()> {
        // Implementation for moving selection up
        Ok(())
    }
    
    async fn move_selection_down(&mut self) -> Result<()> {
        // Implementation for moving selection down
        Ok(())
    }
}

/// Chat input component
pub struct ChatInput {
    state: ComponentState,
    content: String,
    placeholder: String,
}

impl ChatInput {
    pub fn new() -> Self {
        Self {
            state: ComponentState::new(),
            content: String::new(),
            placeholder: "Type your message...".to_string(),
        }
    }
    
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }
    
    pub fn clear(&mut self) {
        self.content.clear();
    }
    
    pub fn get_content(&self) -> &str {
        &self.content
    }
}

#[async_trait]
impl Component for ChatInput {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Char(c) => {
                self.content.push(c);
            }
            KeyCode::Backspace => {
                self.content.pop();
            }
            _ => {}
        }
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Message Input")
            .style(if self.state.has_focus {
                theme.focused_border_style()
            } else {
                theme.border_style()
            });
        
        let display_text = if self.content.is_empty() {
            &self.placeholder
        } else {
            &self.content
        };
        
        let paragraph = Paragraph::new(display_text.as_str())
            .block(block)
            .style(theme.text_style());
        
        frame.render_widget(paragraph, area);
        self.state.size = area;
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

/// Chat session containing multiple messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<ChatMessage>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ChatSession {
    pub fn new(title: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.updated_at = chrono::Utc::now();
    }
    
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}