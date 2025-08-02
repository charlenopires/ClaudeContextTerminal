use super::{Page, PageId};
use crate::tui::{
    components::{
        chat::{ChatInput, ChatMessage, ChatMessageList, ChatSession},
        Component,
    },
    styles::Theme,
    Frame,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
};

/// Chat page for AI conversation interface
pub struct ChatPage {
    id: PageId,
    title: String,
    message_list: ChatMessageList,
    input: ChatInput,
    current_session: Option<ChatSession>,
}

impl ChatPage {
    pub fn new() -> Self {
        let mut message_list = ChatMessageList::new()
            .with_auto_scroll(true)
            .with_timestamps(true);
        
        let mut input = ChatInput::new()
            .with_placeholder("Type your message here...".to_string());
        
        // Set initial focus
        input.set_focus(true);
        
        Self {
            id: "chat".to_string(),
            title: "AI Chat".to_string(),
            message_list,
            input,
            current_session: None,
        }
    }
}

#[async_trait]
impl Page for ChatPage {
    fn id(&self) -> &PageId {
        &self.id
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        match event.code {
            KeyCode::Tab => {
                // Switch focus between components
            }
            KeyCode::Enter => {
                // Send message
                let _content = self.input.get_content().to_string();
                self.input.clear();
            }
            _ => {
                // Forward to focused component
                self.input.handle_key_event(event).await?;
            }
        }
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        self.message_list.handle_mouse_event(event).await?;
        self.input.handle_mouse_event(event).await?;
        Ok(())
    }
    
    async fn tick(&mut self) -> Result<()> {
        self.message_list.tick().await?;
        self.input.tick().await?;
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Messages
                Constraint::Length(3),   // Input
            ])
            .split(area);
        
        // Render message list
        self.message_list.render(frame, chunks[0], theme);
        
        // Render input
        self.input.render(frame, chunks[1], theme);
    }
    
    async fn on_enter(&mut self) -> Result<()> {
        // Initialize with a session if none exists
        if self.current_session.is_none() {
            self.current_session = Some(ChatSession::new("New Chat".to_string()));
        }
        Ok(())
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("Tab", "Switch focus"),
            ("Enter", "Send message"),
            ("↑/↓", "Navigate messages"),
        ]
    }
}

impl Default for ChatPage {
    fn default() -> Self {
        Self::new()
    }
}