//! Enhanced input field with completion support

use super::{
    CompletionContext, CompletionEngine, CompletionEvent, CompletionItem, 
    CompletionList, CompletionMessage, CompletionProvider, ProviderPriority,
    FileProvider, CommandProvider, HistoryProvider, CodeProvider,
};
use crate::tui::{
    components::{Component, ComponentState, TextInput},
    themes::Theme,
    Frame,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame as RatatuiFrame,
};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error};

/// Input field with integrated completion support
pub struct CompletionInput {
    state: ComponentState,
    text: String,
    cursor_position: usize,
    completion_engine: Arc<RwLock<CompletionEngine>>,
    completion_list: CompletionList,
    completion_enabled: bool,
    auto_complete: bool,
    completion_delay_ms: u64,
    last_completion_query: String,
    event_sender: mpsc::UnboundedSender<CompletionEvent>,
    event_receiver: mpsc::UnboundedReceiver<CompletionEvent>,
    working_directory: Option<String>,
    command_context: Option<String>,
    language_context: Option<String>,
    placeholder_text: String,
    multiline: bool,
    max_lines: usize,
    show_cursor: bool,
}

impl CompletionInput {
    /// Create a new completion input
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        let completion_list = CompletionList::new().with_event_sender(event_sender.clone());
        
        let mut engine = CompletionEngine::new();
        
        // Register default providers
        engine.register_provider(Arc::new(FileProvider::new()), ProviderPriority::High);
        engine.register_provider(Arc::new(CommandProvider::new()), ProviderPriority::High);
        engine.register_provider(Arc::new(HistoryProvider::new()), ProviderPriority::Medium);
        engine.register_provider(Arc::new(CodeProvider::new()), ProviderPriority::High);
        
        Self {
            state: ComponentState::new(),
            text: String::new(),
            cursor_position: 0,
            completion_engine: Arc::new(RwLock::new(engine)),
            completion_list,
            completion_enabled: true,
            auto_complete: true,
            completion_delay_ms: 100,
            last_completion_query: String::new(),
            event_sender,
            event_receiver,
            working_directory: None,
            command_context: None,
            language_context: None,
            placeholder_text: "Type to search...".to_string(),
            multiline: false,
            max_lines: 1,
            show_cursor: true,
        }
    }

    /// Enable or disable completions
    pub fn with_completions_enabled(mut self, enabled: bool) -> Self {
        self.completion_enabled = enabled;
        self
    }

    /// Enable or disable automatic completion triggers
    pub fn with_auto_complete(mut self, auto: bool) -> Self {
        self.auto_complete = auto;
        self
    }

    /// Set completion delay in milliseconds
    pub fn with_completion_delay(mut self, delay_ms: u64) -> Self {
        self.completion_delay_ms = delay_ms;
        self
    }

    /// Set working directory for file completions
    pub fn with_working_directory(mut self, dir: String) -> Self {
        self.working_directory = Some(dir);
        self
    }

    /// Set command context for command completions
    pub fn with_command_context(mut self, context: String) -> Self {
        self.command_context = Some(context);
        self
    }

    /// Set language context for code completions
    pub fn with_language_context(mut self, language: String) -> Self {
        self.language_context = Some(language);
        self
    }

    /// Set placeholder text
    pub fn with_placeholder(mut self, placeholder: String) -> Self {
        self.placeholder_text = placeholder;
        self
    }

    /// Enable multiline input
    pub fn with_multiline(mut self, enabled: bool, max_lines: usize) -> Self {
        self.multiline = enabled;
        self.max_lines = max_lines;
        self
    }

    /// Register a custom completion provider
    pub async fn register_provider(&mut self, provider: Arc<dyn CompletionProvider>, priority: ProviderPriority) {
        let mut engine = self.completion_engine.write().await;
        engine.register_provider(provider, priority);
    }

    /// Trigger completion manually
    pub async fn trigger_completion(&mut self) -> Result<()> {
        if !self.completion_enabled {
            return Ok(());
        }

        let context = self.create_completion_context();
        debug!("Triggering completion for context: {:?}", context);

        let engine = self.completion_engine.read().await;
        match engine.get_completions(&context).await {
            Ok(items) => {
                if !items.is_empty() {
                    let position = self.calculate_completion_position();
                    self.completion_list.open(items, position, context.current_word().to_string());
                } else {
                    self.completion_list.close();
                }
            }
            Err(e) => {
                error!("Completion failed: {}", e);
                self.completion_list.close();
            }
        }

        Ok(())
    }

    /// Create completion context from current state
    fn create_completion_context(&self) -> CompletionContext {
        CompletionContext {
            text: self.text.clone(),
            cursor_pos: self.cursor_position,
            working_dir: self.working_directory.clone(),
            command_context: self.command_context.clone(),
            language: self.language_context.clone(),
            max_results: 10,
        }
    }

    /// Calculate position for completion popup
    fn calculate_completion_position(&self) -> Rect {
        // This is a simplified calculation - in a real implementation,
        // you would need to consider the actual text layout and cursor position
        let x = self.state.size.x + self.cursor_position as u16;
        let y = self.state.size.y + 1; // Below the input field
        
        Rect::new(x, y, 0, 0)
    }

    /// Handle completion events
    async fn handle_completion_events(&mut self) {
        while let Ok(event) = self.event_receiver.try_recv() {
            match event {
                CompletionEvent::Selected { item, insert } => {
                    self.insert_completion(&item, insert).await;
                }
                CompletionEvent::Closed => {
                    // Completion list was closed
                    debug!("Completion list closed");
                }
                _ => {
                    // Other events are handled by the completion list
                }
            }
        }
    }

    /// Insert a completion item into the text
    async fn insert_completion(&mut self, item: &CompletionItem, insert_only: bool) {
        let context = self.create_completion_context();
        let current_word = context.current_word();
        
        // Find the start of the current word
        let word_start = self.text[..self.cursor_position]
            .rfind(|c: char| c.is_whitespace() || c == '/' || c == '\\')
            .map(|i| i + 1)
            .unwrap_or(0);

        // Replace the current word with the completion
        let before = &self.text[..word_start];
        let after = &self.text[self.cursor_position..];
        
        self.text = format!("{}{}{}", before, item.value, after);
        self.cursor_position = word_start + item.value.len();

        debug!("Inserted completion: '{}' at position {}", item.value, self.cursor_position);

        if !insert_only {
            self.completion_list.close();
        }
    }

    /// Check if we should trigger auto-completion
    fn should_auto_complete(&self, new_text: &str) -> bool {
        if !self.auto_complete || !self.completion_enabled {
            return false;
        }

        // Don't auto-complete if text is empty or too short
        if new_text.len() < 2 {
            return false;
        }

        // Check if we're typing a word (not in whitespace)
        let current_char = new_text.chars().nth(self.cursor_position.saturating_sub(1));
        if let Some(ch) = current_char {
            ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == '/'
        } else {
            false
        }
    }

    /// Move cursor to the left
    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor to the right
    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to beginning of line
    fn move_cursor_home(&mut self) {
        if self.multiline {
            // Find start of current line
            let line_start = self.text[..self.cursor_position]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            self.cursor_position = line_start;
        } else {
            self.cursor_position = 0;
        }
    }

    /// Move cursor to end of line
    fn move_cursor_end(&mut self) {
        if self.multiline {
            // Find end of current line
            let line_end = self.text[self.cursor_position..]
                .find('\n')
                .map(|i| self.cursor_position + i)
                .unwrap_or(self.text.len());
            self.cursor_position = line_end;
        } else {
            self.cursor_position = self.text.len();
        }
    }

    /// Move cursor up (multiline only)
    fn move_cursor_up(&mut self) {
        if !self.multiline {
            return;
        }

        // Find start of current line
        let current_line_start = self.text[..self.cursor_position]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        if current_line_start > 0 {
            // Find start of previous line
            let prev_line_start = self.text[..current_line_start - 1]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            
            let current_col = self.cursor_position - current_line_start;
            let prev_line_end = current_line_start - 1;
            let prev_line_len = prev_line_end - prev_line_start;
            
            self.cursor_position = prev_line_start + current_col.min(prev_line_len);
        }
    }

    /// Move cursor down (multiline only)
    fn move_cursor_down(&mut self) {
        if !self.multiline {
            return;
        }

        // Find start of current line
        let current_line_start = self.text[..self.cursor_position]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        
        // Find start of next line
        if let Some(next_line_start) = self.text[self.cursor_position..].find('\n') {
            let next_line_start = self.cursor_position + next_line_start + 1;
            
            if next_line_start < self.text.len() {
                let current_col = self.cursor_position - current_line_start;
                
                // Find end of next line
                let next_line_end = self.text[next_line_start..]
                    .find('\n')
                    .map(|i| next_line_start + i)
                    .unwrap_or(self.text.len());
                
                let next_line_len = next_line_end - next_line_start;
                self.cursor_position = next_line_start + current_col.min(next_line_len);
            }
        }
    }

    /// Create the display text with cursor highlighting
    fn create_display_text(&self, theme: &Theme) -> Vec<Line> {
        if self.text.is_empty() && !self.state.has_focus {
            // Show placeholder
            return vec![Line::from(Span::styled(
                &self.placeholder_text,
                Style::default().fg(theme.colors.fg_muted).add_modifier(Modifier::ITALIC),
            ))];
        }

        let mut lines = Vec::new();
        let text_lines: Vec<&str> = if self.multiline {
            self.text.split('\n').collect()
        } else {
            vec![&self.text]
        };

        let mut char_pos = 0;
        for (line_idx, line_text) in text_lines.iter().enumerate() {
            let mut spans = Vec::new();
            let line_start = char_pos;
            let line_end = line_start + line_text.len();

            if self.show_cursor && self.state.has_focus && 
               self.cursor_position >= line_start && self.cursor_position <= line_end {
                // Cursor is on this line
                let cursor_pos_in_line = self.cursor_position - line_start;
                
                if cursor_pos_in_line > 0 {
                    spans.push(Span::raw(&line_text[..cursor_pos_in_line]));
                }
                
                // Cursor character
                if cursor_pos_in_line < line_text.len() {
                    let cursor_char = &line_text[cursor_pos_in_line..cursor_pos_in_line + 1];
                    spans.push(Span::styled(
                        cursor_char,
                        Style::default().bg(theme.colors.accent).fg(theme.colors.bg_base),
                    ));
                    
                    if cursor_pos_in_line + 1 < line_text.len() {
                        spans.push(Span::raw(&line_text[cursor_pos_in_line + 1..]));
                    }
                } else {
                    // Cursor at end of line
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(theme.colors.accent),
                    ));
                }
            } else {
                // No cursor on this line
                spans.push(Span::raw(*line_text));
            }

            lines.push(Line::from(spans));
            char_pos = line_end + 1; // +1 for newline character
            
            if !self.multiline {
                break;
            }
        }

        lines
    }
}

impl Default for CompletionInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for CompletionInput {
    async fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
        // Handle completion events first
        self.handle_completion_events().await;

        // If completion list is visible, let it handle navigation keys
        if self.completion_list.is_visible() {
            match event.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Enter | KeyCode::Tab | KeyCode::Esc => {
                    return self.completion_list.handle_key_event(event).await;
                }
                KeyCode::Char('n') | KeyCode::Char('p') | KeyCode::Char('y') 
                    if event.modifiers.contains(KeyModifiers::CONTROL) => {
                    return self.completion_list.handle_key_event(event).await;
                }
                _ => {
                    // Continue to handle input normally
                }
            }
        }

        // Handle input events
        match event.code {
            KeyCode::Char(c) => {
                self.insert_char(c).await?;
            }
            KeyCode::Backspace => {
                self.delete_previous_char().await?;
            }
            KeyCode::Delete => {
                self.delete_char().await?;
            }
            KeyCode::Left => {
                self.move_cursor_left();
                self.completion_list.close();
            }
            KeyCode::Right => {
                self.move_cursor_right();
                self.completion_list.close();
            }
            KeyCode::Home => {
                self.move_cursor_home();
                self.completion_list.close();
            }
            KeyCode::End => {
                self.move_cursor_end();
                self.completion_list.close();
            }
            KeyCode::Up if self.multiline => {
                self.move_cursor_up();
                self.completion_list.close();
            }
            KeyCode::Down if self.multiline => {
                self.move_cursor_down();
                self.completion_list.close();
            }
            KeyCode::Enter if self.multiline => {
                self.insert_char('\n').await?;
            }
            KeyCode::Tab if self.completion_enabled => {
                self.trigger_completion().await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        // TODO: Implement mouse cursor positioning
        self.completion_list.handle_mouse_event(event).await
    }

    async fn tick(&mut self) -> Result<()> {
        self.handle_completion_events().await;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let input_height = if self.multiline {
            self.max_lines.min(area.height as usize) as u16
        } else {
            1
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(input_height + 2), // +2 for borders
                Constraint::Min(0),
            ])
            .split(area);

        let input_area = chunks[0];

        // Create the input widget
        let display_lines = self.create_display_text(theme);
        let input_widget = Paragraph::new(display_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.state.has_focus {
                        Style::default().fg(theme.colors.accent)
                    } else {
                        Style::default().fg(theme.colors.border)
                    })
                    .title(if self.completion_enabled { "Input (Tab for completions)" } else { "Input" })
                    .title_style(Style::default().fg(theme.colors.fg_base)),
            )
            .style(Style::default().fg(theme.colors.fg_base))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(input_widget, input_area);

        // Render completion list if visible
        if self.completion_list.is_visible() {
            self.completion_list.render(frame, area, theme);
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
        if !focus {
            self.completion_list.close();
        }
    }

    fn is_visible(&self) -> bool {
        self.state.is_visible
    }

    fn set_visible(&mut self, visible: bool) {
        self.state.is_visible = visible;
        if !visible {
            self.completion_list.close();
        }
    }
}

#[async_trait]
impl TextInput for CompletionInput {
    async fn insert_char(&mut self, c: char) -> Result<()> {
        self.text.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();

        // Trigger auto-completion if appropriate
        if self.should_auto_complete(&self.text) {
            // Add a small delay to avoid too frequent completions
            tokio::time::sleep(tokio::time::Duration::from_millis(self.completion_delay_ms)).await;
            self.trigger_completion().await?;
        }

        Ok(())
    }

    async fn delete_char(&mut self) -> Result<()> {
        if self.cursor_position < self.text.len() {
            self.text.remove(self.cursor_position);
            self.completion_list.close();
        }
        Ok(())
    }

    async fn delete_previous_char(&mut self) -> Result<()> {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.text.remove(self.cursor_position);
            self.completion_list.close();
        }
        Ok(())
    }

    fn get_text(&self) -> &str {
        &self.text
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
        self.cursor_position = self.text.len();
        self.completion_list.close();
    }

    fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    fn set_cursor_position(&mut self, pos: usize) {
        self.cursor_position = pos.min(self.text.len());
    }
}