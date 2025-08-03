//! Model selection dialog
//! 
//! This dialog allows users to view and select available AI models
//! for their conversations.

use super::types::{Dialog, DialogConfig, DialogId, DialogPosition, DialogSize, dialog_ids};
use crate::{
    config::Config,
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
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub description: Option<String>,
    pub context_length: Option<u32>,
    pub is_available: bool,
    pub requires_api_key: bool,
}

impl ModelInfo {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
            description: None,
            context_length: None,
            is_available: true,
            requires_api_key: false,
        }
    }
    
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    pub fn with_context_length(mut self, length: u32) -> Self {
        self.context_length = Some(length);
        self
    }
    
    pub fn with_availability(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }
    
    pub fn requires_api_key(mut self, requires: bool) -> Self {
        self.requires_api_key = requires;
        self
    }
}

/// Models dialog for selecting AI models
pub struct ModelsDialog {
    /// Component state
    state: ComponentState,
    
    /// Dialog configuration
    config: DialogConfig,
    
    /// List of available models
    models: Vec<ModelInfo>,
    
    /// List state for navigation
    list_state: ListState,
    
    /// Event sender for dialog events
    event_sender: Option<mpsc::UnboundedSender<Event>>,
    
    /// Current configuration
    current_config: Option<Config>,
    
    /// Currently selected model ID
    current_model: Option<String>,
    
    /// Filter text for searching models
    filter_text: String,
    
    /// Whether we're in filter/search mode
    in_search_mode: bool,
    
    /// Loading state
    is_loading: bool,
    
    /// Error message if any
    error_message: Option<String>,
}

impl ModelsDialog {
    /// Create a new models dialog
    pub fn new() -> Self {
        let config = DialogConfig::new(dialog_ids::models())
            .with_title("Select Model".to_string())
            .with_position(DialogPosition::Center)
            .with_size(DialogSize::Percentage(60, 70))
            .with_border(true)
            .modal(true)
            .closable(true);
        
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        
        Self {
            state: ComponentState::new(),
            config,
            models: Vec::new(),
            list_state,
            event_sender: None,
            current_config: None,
            current_model: None,
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
    
    /// Set the current configuration
    pub fn set_config(&mut self, config: Config) {
        self.current_model = Some(config.model.clone());
        self.current_config = Some(config);
    }
    
    /// Load available models
    pub async fn load_models(&mut self) -> Result<()> {
        self.is_loading = true;
        self.error_message = None;
        
        // Create a basic set of known models
        self.models = vec![
            // OpenAI Models
            ModelInfo::new("gpt-4", "GPT-4", "openai")
                .with_description("Most capable GPT-4 model")
                .with_context_length(8192)
                .requires_api_key(true),
                
            ModelInfo::new("gpt-4-turbo", "GPT-4 Turbo", "openai")
                .with_description("Latest GPT-4 model with improved capabilities")
                .with_context_length(128000)
                .requires_api_key(true),
                
            ModelInfo::new("gpt-3.5-turbo", "GPT-3.5 Turbo", "openai")
                .with_description("Fast and efficient ChatGPT model")
                .with_context_length(4096)
                .requires_api_key(true),
                
            // Anthropic Models
            ModelInfo::new("claude-3-opus-20240229", "Claude 3 Opus", "anthropic")
                .with_description("Most powerful Claude model")
                .with_context_length(200000)
                .requires_api_key(true),
                
            ModelInfo::new("claude-3-sonnet-20240229", "Claude 3 Sonnet", "anthropic")
                .with_description("Balanced Claude model")
                .with_context_length(200000)
                .requires_api_key(true),
                
            ModelInfo::new("claude-3-haiku-20240307", "Claude 3 Haiku", "anthropic")
                .with_description("Fast Claude model")
                .with_context_length(200000)
                .requires_api_key(true),
                
            // Ollama Models (examples)
            ModelInfo::new("llama3.2", "Llama 3.2", "ollama")
                .with_description("Meta's Llama 3.2 model")
                .with_context_length(8192),
                
            ModelInfo::new("codellama", "Code Llama", "ollama")
                .with_description("Specialized coding model")
                .with_context_length(16384),
                
            ModelInfo::new("mistral", "Mistral", "ollama")
                .with_description("Mistral AI model")
                .with_context_length(8192),
        ];
        
        // Set current selection to the current model if it exists
        if let Some(current) = &self.current_model {
            if let Some(index) = self.models.iter().position(|m| &m.id == current) {
                self.list_state.select(Some(index));
            }
        }
        
        self.is_loading = false;
        Ok(())
    }
    
    /// Select the currently highlighted model
    pub async fn select_model(&self) -> Result<()> {
        if let Some(index) = self.list_state.selected() {
            if let Some(model) = self.filtered_models().get(index) {
                if let Some(sender) = &self.event_sender {
                    let _ = sender.send(Event::Custom(
                        "model_selected".to_string(),
                        serde_json::json!({
                            "model_id": model.id,
                            "provider": model.provider
                        }),
                    ));
                }
                self.close_dialog().await?;
            }
        }
        Ok(())
    }
    
    /// Get filtered models based on search text
    fn filtered_models(&self) -> Vec<&ModelInfo> {
        if self.filter_text.is_empty() {
            self.models.iter().collect()
        } else {
            self.models
                .iter()
                .filter(|model| {
                    model.name.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || model.id.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || model.provider.to_lowercase().contains(&self.filter_text.to_lowercase())
                        || model.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&self.filter_text.to_lowercase()))
                            .unwrap_or(false)
                })
                .collect()
        }
    }
    
    /// Move selection up
    fn move_selection_up(&mut self) {
        let filtered_count = self.filtered_models().len();
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
        let filtered_count = self.filtered_models().len();
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
    
    /// Render the model list
    fn render_model_list(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let filtered_models = self.filtered_models();
        
        if self.is_loading {
            let loading = Paragraph::new("Loading models...")
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(loading, area);
            return;
        }
        
        if filtered_models.is_empty() {
            let empty_msg = if self.filter_text.is_empty() {
                "No models available."
            } else {
                "No models match your search."
            };
            
            let empty = Paragraph::new(empty_msg)
                .style(Style::default().fg(theme.text_muted()))
                .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }
        
        let items: Vec<ListItem> = filtered_models
            .iter()
            .map(|model| {
                let mut line = format!("{} ({})", model.name, model.provider);
                
                // Add current indicator
                if let Some(current) = &self.current_model {
                    if &model.id == current {
                        line = format!("● {}", line);
                    } else {
                        line = format!("  {}", line);
                    }
                }
                
                // Add availability indicator
                if !model.is_available {
                    line = format!("{} [UNAVAILABLE]", line);
                }
                
                // Add description
                if let Some(desc) = &model.description {
                    line = format!("{}\n    {}", line, desc);
                }
                
                // Add context length
                if let Some(context) = model.context_length {
                    line = format!("{}\n    Context: {} tokens", line, context);
                }
                
                let style = if model.is_available {
                    Style::default().fg(theme.text)
                } else {
                    Style::default().fg(theme.text_muted())
                };
                
                ListItem::new(line).style(style)
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
            "Press '/' to search models..."
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
            "↑/↓: Navigate • Enter: Select • /: Search • Esc: Close"
        };
        
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(theme.text_muted()).add_modifier(Modifier::DIM))
            .alignment(Alignment::Center);
        
        frame.render_widget(help, area);
    }
}

#[async_trait]
impl Component for ModelsDialog {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        if self.in_search_mode {
            match event.code {
                KeyCode::Esc => {
                    self.in_search_mode = false;
                }
                KeyCode::Enter => {
                    self.in_search_mode = false;
                    // Apply filter
                    if self.list_state.selected().is_none() && !self.filtered_models().is_empty() {
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
                    self.select_model().await?;
                }
                
                // Search
                (KeyCode::Char('/'), _) => {
                    self.in_search_mode = true;
                    self.filter_text.clear();
                }
                
                // Refresh
                (KeyCode::Char('r') | KeyCode::Char('R'), _) => {
                    self.load_models().await?;
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
impl Dialog for ModelsDialog {
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
        let width = (available_area.width as f32 * 0.6) as u16;
        let height = (available_area.height as f32 * 0.7) as u16;
        (width.max(50), height.max(15))
    }
    
    async fn on_open(&mut self) -> Result<()> {
        self.load_models().await?;
        Ok(())
    }
    
    fn render_content(&mut self, frame: &mut Frame, content_area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),   // Search bar
                Constraint::Min(5),      // Model list
                Constraint::Length(1),   // Help text
            ])
            .split(content_area);
        
        // Render search bar
        self.render_search_bar(frame, chunks[0], theme);
        
        // Render model list
        self.render_model_list(frame, chunks[1], theme);
        
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
        (60, 20)
    }
}

impl Default for ModelsDialog {
    fn default() -> Self {
        Self::new()
    }
}