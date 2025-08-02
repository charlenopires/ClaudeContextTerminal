use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Key binding configuration
#[derive(Debug, Clone)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub description: String,
}

impl KeyBinding {
    pub fn new(key: KeyCode, modifiers: KeyModifiers, description: &str) -> Self {
        Self {
            key,
            modifiers,
            description: description.to_string(),
        }
    }
    
    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.code && self.modifiers == event.modifiers
    }
}

/// Application key mappings
#[derive(Debug, Clone)]
pub struct KeyMap {
    /// Quit application
    pub quit: KeyBinding,
    
    /// Show help
    pub help: KeyBinding,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            quit: KeyBinding::new(
                KeyCode::Char('c'),
                KeyModifiers::CONTROL,
                "Quit application"
            ),
            help: KeyBinding::new(
                KeyCode::Char('g'),
                KeyModifiers::CONTROL,
                "Show/hide help"
            ),
        }
    }
}

impl KeyMap {
    /// Check if the event should quit the application
    pub fn should_quit(&self, event: &KeyEvent) -> bool {
        self.quit.matches(event)
    }
    
    /// Check if the event should show help
    pub fn should_show_help(&self, event: &KeyEvent) -> bool {
        self.help.matches(event)
    }
    
    /// Get help text for all key bindings
    pub fn help_text(&self) -> String {
        format!("{}\n{}", self.quit.description, self.help.description)
    }
}