use super::{Page, PageId};
use crate::tui::{styles::Theme, Frame};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

/// Home/Welcome page
pub struct HomePage {
    id: PageId,
    title: String,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            id: "home".to_string(),
            title: "Home".to_string(),
        }
    }
}

#[async_trait]
impl Page for HomePage {
    fn id(&self) -> &PageId {
        &self.id
    }
    
    fn title(&self) -> &str {
        &self.title
    }
    
    async fn handle_key_event(&mut self, _event: KeyEvent) -> Result<()> {
        Ok(())
    }
    
    async fn handle_mouse_event(&mut self, _event: MouseEvent) -> Result<()> {
        Ok(())
    }
    
    async fn tick(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let welcome_text = "Welcome to Crush Terminal\n\nPress Enter to start chatting!";
        
        let paragraph = Paragraph::new(welcome_text)
            .block(Block::default().borders(Borders::ALL).title("Welcome"))
            .style(theme.text_style());
        
        frame.render_widget(paragraph, area);
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("Enter", "Start chat"),
            ("Esc", "Exit"),
        ]
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self::new()
    }
}