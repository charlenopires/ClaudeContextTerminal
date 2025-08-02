use super::{Page, PageId};
use crate::tui::{styles::Theme, Frame};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
};

/// Settings page for application configuration
pub struct SettingsPage {
    id: PageId,
    title: String,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            id: "settings".to_string(),
            title: "Settings".to_string(),
        }
    }
}

#[async_trait]
impl Page for SettingsPage {
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
        let settings_text = "Settings\n\nComing soon...";
        
        let paragraph = Paragraph::new(settings_text)
            .block(Block::default().borders(Borders::ALL).title("Settings"))
            .style(theme.text_style());
        
        frame.render_widget(paragraph, area);
    }
    
    fn help_text(&self) -> Vec<(&str, &str)> {
        vec![
            ("Esc", "Go back"),
        ]
    }
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self::new()
    }
}