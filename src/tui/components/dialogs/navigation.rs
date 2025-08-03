//! Dialog navigation system for keyboard-based dialog management
//! 
//! This module handles navigation between dialogs, including focus management,
//! keyboard shortcuts, and dialog switching.

use super::types::{DialogId, DialogEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Navigation actions that can be performed on dialogs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    /// Navigate to the next dialog in the stack
    NextDialog,
    /// Navigate to the previous dialog in the stack
    PreviousDialog,
    /// Close the current dialog
    CloseDialog,
    /// Close all dialogs
    CloseAllDialogs,
    /// Open a specific dialog
    OpenDialog(DialogId),
    /// Toggle a specific dialog (open if closed, close if open)
    ToggleDialog(DialogId),
    /// Bring dialog to front
    BringToFront(DialogId),
    /// Send dialog to back
    SendToBack(DialogId),
    /// Minimize/restore dialog
    MinimizeDialog(DialogId),
    /// Maximize/restore dialog
    MaximizeDialog(DialogId),
}

/// Key binding for navigation actions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(key_code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key_code, modifiers }
    }
    
    pub fn from_key_event(event: KeyEvent) -> Self {
        Self {
            key_code: event.code,
            modifiers: event.modifiers,
        }
    }
    
    pub fn matches(&self, event: KeyEvent) -> bool {
        self.key_code == event.code && self.modifiers == event.modifiers
    }
}

/// Dialog navigation manager
#[derive(Debug)]
pub struct DialogNavigation {
    /// Key bindings for navigation actions
    key_bindings: HashMap<KeyBinding, NavigationAction>,
    
    /// Whether navigation is enabled
    enabled: bool,
    
    /// Navigation history for back/forward functionality
    history: Vec<DialogId>,
    
    /// Current position in history
    history_position: usize,
    
    /// Maximum history length
    max_history: usize,
}

impl DialogNavigation {
    /// Create a new dialog navigation manager
    pub fn new() -> Self {
        let mut navigation = Self {
            key_bindings: HashMap::new(),
            enabled: true,
            history: Vec::new(),
            history_position: 0,
            max_history: 50,
        };
        
        navigation.setup_default_bindings();
        navigation
    }
    
    /// Set up default key bindings
    fn setup_default_bindings(&mut self) {
        // Close current dialog
        self.bind_key(
            KeyCode::Esc,
            KeyModifiers::NONE,
            NavigationAction::CloseDialog,
        );
        
        // Close all dialogs
        self.bind_key(
            KeyCode::Esc,
            KeyModifiers::SHIFT,
            NavigationAction::CloseAllDialogs,
        );
        
        // Navigate between dialogs
        self.bind_key(
            KeyCode::Tab,
            KeyModifiers::NONE,
            NavigationAction::NextDialog,
        );
        
        self.bind_key(
            KeyCode::BackTab,
            KeyModifiers::SHIFT,
            NavigationAction::PreviousDialog,
        );
        
        // Alternative navigation with Ctrl+Tab
        self.bind_key(
            KeyCode::Tab,
            KeyModifiers::CONTROL,
            NavigationAction::NextDialog,
        );
        
        self.bind_key(
            KeyCode::Tab,
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            NavigationAction::PreviousDialog,
        );
        
        // Arrow key navigation
        self.bind_key(
            KeyCode::Right,
            KeyModifiers::ALT,
            NavigationAction::NextDialog,
        );
        
        self.bind_key(
            KeyCode::Left,
            KeyModifiers::ALT,
            NavigationAction::PreviousDialog,
        );
    }
    
    /// Bind a key to a navigation action
    pub fn bind_key(
        &mut self,
        key_code: KeyCode,
        modifiers: KeyModifiers,
        action: NavigationAction,
    ) {
        let binding = KeyBinding::new(key_code, modifiers);
        self.key_bindings.insert(binding, action);
    }
    
    /// Remove a key binding
    pub fn unbind_key(&mut self, key_code: KeyCode, modifiers: KeyModifiers) {
        let binding = KeyBinding::new(key_code, modifiers);
        self.key_bindings.remove(&binding);
    }
    
    /// Get the navigation action for a key event
    pub fn get_action(&self, event: KeyEvent) -> Option<&NavigationAction> {
        if !self.enabled {
            return None;
        }
        
        let binding = KeyBinding::from_key_event(event);
        self.key_bindings.get(&binding)
    }
    
    /// Check if a key event is handled by navigation
    pub fn handles_key(&self, event: KeyEvent) -> bool {
        self.get_action(event).is_some()
    }
    
    /// Enable or disable navigation
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Check if navigation is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Add a dialog to navigation history
    pub fn add_to_history(&mut self, dialog_id: DialogId) {
        // Remove existing occurrence if present
        self.history.retain(|id| id != &dialog_id);
        
        // Add to front of history
        self.history.insert(0, dialog_id);
        
        // Trim history to max length
        if self.history.len() > self.max_history {
            self.history.truncate(self.max_history);
        }
        
        self.history_position = 0;
    }
    
    /// Remove a dialog from navigation history
    pub fn remove_from_history(&mut self, dialog_id: &DialogId) {
        let initial_len = self.history.len();
        self.history.retain(|id| id != dialog_id);
        
        // Adjust position if items were removed before it
        if self.history.len() < initial_len && self.history_position > 0 {
            self.history_position = self.history_position.saturating_sub(1);
        }
        
        // Ensure position is valid
        if self.history_position >= self.history.len() && !self.history.is_empty() {
            self.history_position = self.history.len() - 1;
        }
    }
    
    /// Get the current dialog from history
    pub fn current_from_history(&self) -> Option<&DialogId> {
        self.history.get(self.history_position)
    }
    
    /// Navigate to previous dialog in history
    pub fn previous_in_history(&mut self) -> Option<&DialogId> {
        if self.history_position + 1 < self.history.len() {
            self.history_position += 1;
            self.history.get(self.history_position)
        } else {
            None
        }
    }
    
    /// Navigate to next dialog in history
    pub fn next_in_history(&mut self) -> Option<&DialogId> {
        if self.history_position > 0 {
            self.history_position -= 1;
            self.history.get(self.history_position)
        } else {
            None
        }
    }
    
    /// Clear navigation history
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_position = 0;
    }
    
    /// Get all key bindings
    pub fn key_bindings(&self) -> &HashMap<KeyBinding, NavigationAction> {
        &self.key_bindings
    }
    
    /// Set maximum history length
    pub fn set_max_history(&mut self, max_history: usize) {
        self.max_history = max_history;
        
        // Trim current history if needed
        if self.history.len() > max_history {
            self.history.truncate(max_history);
            
            // Adjust position if it's now out of bounds
            if self.history_position >= self.history.len() && !self.history.is_empty() {
                self.history_position = self.history.len() - 1;
            }
        }
    }
    
    /// Get navigation help text
    pub fn help_text(&self) -> Vec<(String, String)> {
        let mut help = Vec::new();
        
        for (binding, action) in &self.key_bindings {
            let key_text = format_key_binding(binding);
            let action_text = format_action(action);
            help.push((key_text, action_text));
        }
        
        // Sort by action type for better organization
        help.sort_by(|a, b| a.1.cmp(&b.1));
        
        help
    }
}

impl Default for DialogNavigation {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a key binding for display
fn format_key_binding(binding: &KeyBinding) -> String {
    let mut parts = Vec::new();
    
    if binding.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if binding.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if binding.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    
    let key_name = match binding.key_code {
        KeyCode::Esc => "Esc",
        KeyCode::Tab => "Tab",
        KeyCode::BackTab => "Shift+Tab",
        KeyCode::Enter => "Enter",
        KeyCode::Left => "←",
        KeyCode::Right => "→",
        KeyCode::Up => "↑",
        KeyCode::Down => "↓",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PgUp",
        KeyCode::PageDown => "PgDn",
        KeyCode::Delete => "Del",
        KeyCode::Insert => "Ins",
        KeyCode::F(n) => return format!("F{}", n),
        KeyCode::Char(c) => return c.to_string().to_uppercase(),
        _ => "Unknown",
    };
    
    parts.push(key_name);
    parts.join("+")
}

/// Format a navigation action for display
fn format_action(action: &NavigationAction) -> String {
    match action {
        NavigationAction::NextDialog => "Next Dialog".to_string(),
        NavigationAction::PreviousDialog => "Previous Dialog".to_string(),
        NavigationAction::CloseDialog => "Close Dialog".to_string(),
        NavigationAction::CloseAllDialogs => "Close All Dialogs".to_string(),
        NavigationAction::OpenDialog(id) => format!("Open {}", id),
        NavigationAction::ToggleDialog(id) => format!("Toggle {}", id),
        NavigationAction::BringToFront(id) => format!("Bring {} to Front", id),
        NavigationAction::SendToBack(id) => format!("Send {} to Back", id),
        NavigationAction::MinimizeDialog(id) => format!("Minimize {}", id),
        NavigationAction::MaximizeDialog(id) => format!("Maximize {}", id),
    }
}

/// Predefined navigation shortcuts for common dialog operations
pub mod shortcuts {
    use super::*;
    
    /// Create key binding for Escape key
    pub fn escape() -> KeyBinding {
        KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE)
    }
    
    /// Create key binding for Shift+Escape
    pub fn shift_escape() -> KeyBinding {
        KeyBinding::new(KeyCode::Esc, KeyModifiers::SHIFT)
    }
    
    /// Create key binding for Tab
    pub fn tab() -> KeyBinding {
        KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE)
    }
    
    /// Create key binding for Shift+Tab
    pub fn shift_tab() -> KeyBinding {
        KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT)
    }
    
    /// Create key binding for Ctrl+Tab
    pub fn ctrl_tab() -> KeyBinding {
        KeyBinding::new(KeyCode::Tab, KeyModifiers::CONTROL)
    }
    
    /// Create key binding for Ctrl+Shift+Tab
    pub fn ctrl_shift_tab() -> KeyBinding {
        KeyBinding::new(KeyCode::Tab, KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    }
    
    /// Create key binding for Alt+Left
    pub fn alt_left() -> KeyBinding {
        KeyBinding::new(KeyCode::Left, KeyModifiers::ALT)
    }
    
    /// Create key binding for Alt+Right
    pub fn alt_right() -> KeyBinding {
        KeyBinding::new(KeyCode::Right, KeyModifiers::ALT)
    }
}