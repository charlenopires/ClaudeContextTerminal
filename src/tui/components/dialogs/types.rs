//! Core dialog types and traits
//! 
//! This module defines the fundamental types and traits for the dialog system,
//! providing a foundation for modal UI components.

use crate::tui::{
    components::Component,
    events::Event,
    themes::Theme,
    Frame,
};
use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for dialog instances
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DialogId(pub String);

impl DialogId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DialogId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl From<String> for DialogId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for DialogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Dialog positioning options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogPosition {
    /// Center the dialog in the available area
    Center,
    /// Position at specific coordinates (row, col)
    Fixed(u16, u16),
    /// Position relative to another component
    Relative { offset_x: i16, offset_y: i16 },
    /// Position at top of screen
    Top,
    /// Position at bottom of screen
    Bottom,
    /// Position at left side of screen
    Left,
    /// Position at right side of screen
    Right,
}

impl Default for DialogPosition {
    fn default() -> Self {
        Self::Center
    }
}

/// Dialog size options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogSize {
    /// Fixed size in characters (width, height)
    Fixed(u16, u16),
    /// Percentage of available area (width_pct, height_pct)
    Percentage(u16, u16),
    /// Fit content with optional minimum size
    FitContent { min_width: u16, min_height: u16 },
    /// Full screen
    FullScreen,
}

impl Default for DialogSize {
    fn default() -> Self {
        Self::FitContent {
            min_width: 40,
            min_height: 10,
        }
    }
}

/// Dialog animation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAnimation {
    /// No animation
    None,
    /// Fade in/out
    Fade,
    /// Slide from direction
    Slide(SlideDirection),
    /// Scale in/out
    Scale,
    /// Bounce effect
    Bounce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

impl Default for DialogAnimation {
    fn default() -> Self {
        Self::Fade
    }
}

/// Dialog configuration options
#[derive(Debug, Clone)]
pub struct DialogConfig {
    /// Dialog identifier
    pub id: DialogId,
    /// Dialog title (optional)
    pub title: Option<String>,
    /// Position configuration
    pub position: DialogPosition,
    /// Size configuration
    pub size: DialogSize,
    /// Animation configuration
    pub animation: DialogAnimation,
    /// Whether dialog is modal (blocks interaction with other components)
    pub modal: bool,
    /// Whether dialog can be closed with Escape key
    pub closable: bool,
    /// Whether dialog should be destroyed when closed
    pub destroy_on_close: bool,
    /// Whether dialog has a border
    pub has_border: bool,
    /// Whether dialog has a shadow
    pub has_shadow: bool,
    /// Custom CSS-like styling properties
    pub styles: HashMap<String, String>,
    /// Z-index for layering (higher values appear on top)
    pub z_index: i32,
}

impl DialogConfig {
    pub fn new(id: impl Into<DialogId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            position: DialogPosition::default(),
            size: DialogSize::default(),
            animation: DialogAnimation::default(),
            modal: true,
            closable: true,
            destroy_on_close: true,
            has_border: true,
            has_shadow: false,
            styles: HashMap::new(),
            z_index: 100,
        }
    }
    
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    
    pub fn with_position(mut self, position: DialogPosition) -> Self {
        self.position = position;
        self
    }
    
    pub fn with_size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }
    
    pub fn with_animation(mut self, animation: DialogAnimation) -> Self {
        self.animation = animation;
        self
    }
    
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }
    
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
    
    pub fn with_border(mut self, has_border: bool) -> Self {
        self.has_border = has_border;
        self
    }
    
    pub fn with_shadow(mut self, has_shadow: bool) -> Self {
        self.has_shadow = has_shadow;
        self
    }
    
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
    
    pub fn with_style(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.styles.insert(key.into(), value.into());
        self
    }
}

/// Dialog events that can be emitted by dialogs
#[derive(Debug, Clone)]
pub enum DialogEvent {
    /// Dialog was opened
    Opened(DialogId),
    /// Dialog was closed
    Closed(DialogId),
    /// Dialog received focus
    Focused(DialogId),
    /// Dialog lost focus
    Blurred(DialogId),
    /// Dialog was resized
    Resized(DialogId, Rect),
    /// Dialog specific custom event
    Custom(DialogId, String, serde_json::Value),
    /// Request to close dialog
    RequestClose(DialogId),
    /// Request to open dialog (dialog will be created by the handler)
    RequestOpen(DialogId),
}

/// Result type for dialog operations
pub type DialogResult<T> = std::result::Result<T, DialogError>;

/// Dialog-specific error types
#[derive(Debug, thiserror::Error)]
pub enum DialogError {
    #[error("Dialog with ID '{0}' not found")]
    NotFound(DialogId),
    
    #[error("Dialog with ID '{0}' already exists")]
    AlreadyExists(DialogId),
    
    #[error("Invalid dialog configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Dialog operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Dialog component error: {0}")]
    ComponentError(#[from] anyhow::Error),
}

/// Callback trait for dialog lifecycle events
#[async_trait]
pub trait DialogCallback: Send + Sync {
    /// Called when dialog is about to open
    async fn on_opening(&mut self, dialog_id: &DialogId) -> Result<()> {
        let _ = dialog_id;
        Ok(())
    }
    
    /// Called when dialog has been opened
    async fn on_opened(&mut self, dialog_id: &DialogId) -> Result<()> {
        let _ = dialog_id;
        Ok(())
    }
    
    /// Called when dialog is about to close
    async fn on_closing(&mut self, dialog_id: &DialogId) -> Result<bool> {
        let _ = dialog_id;
        Ok(true) // Allow close by default
    }
    
    /// Called when dialog has been closed
    async fn on_closed(&mut self, dialog_id: &DialogId) -> Result<()> {
        let _ = dialog_id;
        Ok(())
    }
    
    /// Called when dialog receives focus
    async fn on_focused(&mut self, dialog_id: &DialogId) -> Result<()> {
        let _ = dialog_id;
        Ok(())
    }
    
    /// Called when dialog loses focus
    async fn on_blurred(&mut self, dialog_id: &DialogId) -> Result<()> {
        let _ = dialog_id;
        Ok(())
    }
}

/// Core trait for dialog components
/// 
/// This trait extends the base Component trait with dialog-specific functionality
/// such as positioning, configuration, and lifecycle management.
#[async_trait]
pub trait Dialog: Component {
    /// Get the dialog's configuration
    fn config(&self) -> &DialogConfig;
    
    /// Get mutable reference to dialog's configuration
    fn config_mut(&mut self) -> &mut DialogConfig;
    
    /// Get the dialog's ID
    fn id(&self) -> &DialogId {
        &self.config().id
    }
    
    /// Get the dialog's preferred position
    fn position(&self, available_area: Rect) -> (u16, u16);
    
    /// Get the dialog's calculated size
    fn dialog_size(&self, available_area: Rect) -> (u16, u16);
    
    /// Check if the dialog is modal
    fn is_modal(&self) -> bool {
        self.config().modal
    }
    
    /// Check if the dialog can be closed with Escape
    fn is_closable(&self) -> bool {
        self.config().closable
    }
    
    /// Handle dialog-specific events
    async fn handle_dialog_event(&mut self, event: DialogEvent) -> Result<()> {
        let _ = event;
        Ok(())
    }
    
    /// Called when dialog is opened
    async fn on_open(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called when dialog is closed
    async fn on_close(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Called to validate if dialog can be closed
    async fn can_close(&self) -> Result<bool> {
        Ok(true)
    }
    
    /// Get dialog's minimum size requirements
    fn min_size(&self) -> (u16, u16) {
        (20, 5)
    }
    
    /// Get dialog's preferred size
    fn preferred_size(&self) -> (u16, u16) {
        (40, 10)
    }
    
    /// Get dialog's maximum size (None = no limit)
    fn max_size(&self) -> Option<(u16, u16)> {
        None
    }
    
    /// Render dialog content (without border/chrome)
    fn render_content(&mut self, frame: &mut Frame, content_area: Rect, theme: &Theme);
    
    /// Render dialog border and chrome (title bar, etc.)
    fn render_chrome(&mut self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if !self.config().has_border {
            return;
        }
        
        use ratatui::{
            style::{Style, Stylize},
            widgets::{Block, Borders},
        };
        
        let mut block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(theme.border));
        
        if let Some(title) = &self.config().title {
            block = block.title(title.clone());
        }
        
        frame.render_widget(block, area);
    }
    
    /// Handle key events specifically for dialogs
    async fn handle_dialog_key(&mut self, key: KeyEvent) -> Result<bool> {
        use crossterm::event::{KeyCode, KeyModifiers};
        
        // Handle escape key for closable dialogs
        if self.is_closable() && key.code == KeyCode::Esc && key.modifiers.is_empty() {
            // Request close - this will be handled by the dialog manager
            return Ok(true); // Indicate we handled the event
        }
        
        Ok(false) // Indicate we didn't handle the event
    }
}

/// Dialog state tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogState {
    /// Dialog is being created/initialized
    Initializing,
    /// Dialog is opening (potentially animating)
    Opening,
    /// Dialog is open and active
    Open,
    /// Dialog is closing (potentially animating)
    Closing,
    /// Dialog is closed/destroyed
    Closed,
}

impl Default for DialogState {
    fn default() -> Self {
        Self::Initializing
    }
}

/// Helper struct for dialog layout calculations
#[derive(Debug, Clone)]
pub struct DialogLayout {
    /// Full available area
    pub available_area: Rect,
    /// Dialog area (including border)
    pub dialog_area: Rect,
    /// Content area (excluding border)
    pub content_area: Rect,
    /// Position coordinates
    pub position: (u16, u16),
    /// Calculated size
    pub size: (u16, u16),
}

impl DialogLayout {
    pub fn calculate(
        config: &DialogConfig,
        available_area: Rect,
        content_size: Option<(u16, u16)>,
    ) -> Self {
        let (width, height) = Self::calculate_size(config, available_area, content_size);
        let (x, y) = Self::calculate_position(config, available_area, width, height);
        
        let dialog_area = Rect {
            x,
            y,
            width,
            height,
        };
        
        let content_area = if config.has_border {
            Rect {
                x: dialog_area.x + 1,
                y: dialog_area.y + 1,
                width: dialog_area.width.saturating_sub(2),
                height: dialog_area.height.saturating_sub(2),
            }
        } else {
            dialog_area
        };
        
        Self {
            available_area,
            dialog_area,
            content_area,
            position: (x, y),
            size: (width, height),
        }
    }
    
    fn calculate_size(
        config: &DialogConfig,
        available_area: Rect,
        content_size: Option<(u16, u16)>,
    ) -> (u16, u16) {
        match config.size {
            DialogSize::Fixed(w, h) => (w, h),
            DialogSize::Percentage(w_pct, h_pct) => {
                let width = (available_area.width as f32 * w_pct as f32 / 100.0) as u16;
                let height = (available_area.height as f32 * h_pct as f32 / 100.0) as u16;
                (width, height)
            }
            DialogSize::FitContent { min_width, min_height } => {
                if let Some((content_w, content_h)) = content_size {
                    let width = content_w.max(min_width);
                    let height = content_h.max(min_height);
                    
                    // Add border size if needed
                    if config.has_border {
                        (width + 2, height + 2)
                    } else {
                        (width, height)
                    }
                } else {
                    (min_width, min_height)
                }
            }
            DialogSize::FullScreen => (available_area.width, available_area.height),
        }
    }
    
    fn calculate_position(
        config: &DialogConfig,
        available_area: Rect,
        width: u16,
        height: u16,
    ) -> (u16, u16) {
        match config.position {
            DialogPosition::Center => {
                let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
                let y = available_area.y + (available_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
            DialogPosition::Fixed(x, y) => (
                available_area.x + x.min(available_area.width.saturating_sub(width)),
                available_area.y + y.min(available_area.height.saturating_sub(height)),
            ),
            DialogPosition::Relative { offset_x, offset_y } => {
                let base_x = available_area.x + available_area.width / 2;
                let base_y = available_area.y + available_area.height / 2;
                
                let x = if offset_x >= 0 {
                    base_x.saturating_add(offset_x as u16)
                } else {
                    base_x.saturating_sub((-offset_x) as u16)
                };
                
                let y = if offset_y >= 0 {
                    base_y.saturating_add(offset_y as u16)
                } else {
                    base_y.saturating_sub((-offset_y) as u16)
                };
                
                (x, y)
            }
            DialogPosition::Top => {
                let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
                (x, available_area.y)
            }
            DialogPosition::Bottom => {
                let x = available_area.x + (available_area.width.saturating_sub(width)) / 2;
                let y = available_area.y + available_area.height.saturating_sub(height);
                (x, y)
            }
            DialogPosition::Left => {
                let y = available_area.y + (available_area.height.saturating_sub(height)) / 2;
                (available_area.x, y)
            }
            DialogPosition::Right => {
                let x = available_area.x + available_area.width.saturating_sub(width);
                let y = available_area.y + (available_area.height.saturating_sub(height)) / 2;
                (x, y)
            }
        }
    }
}

/// Predefined dialog IDs for common dialogs
pub mod dialog_ids {
    use super::DialogId;
    
    pub fn quit() -> DialogId { DialogId("quit".to_string()) }
    pub fn commands() -> DialogId { DialogId("commands".to_string()) }
    pub fn sessions() -> DialogId { DialogId("sessions".to_string()) }
    pub fn models() -> DialogId { DialogId("models".to_string()) }
    pub fn file_picker() -> DialogId { DialogId("file_picker".to_string()) }
    pub fn permissions() -> DialogId { DialogId("permissions".to_string()) }
    pub fn help() -> DialogId { DialogId("help".to_string()) }
    pub fn settings() -> DialogId { DialogId("settings".to_string()) }
    
    pub const QUIT: &str = "quit";
    pub const COMMANDS: &str = "commands";
    pub const SESSIONS: &str = "sessions";
    pub const MODELS: &str = "models";
    pub const FILE_PICKER: &str = "file_picker";
    pub const PERMISSIONS: &str = "permissions";
    pub const HELP: &str = "help";
    pub const SETTINGS: &str = "settings";
}