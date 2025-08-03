//! Completion framework for intelligent auto-completion
//!
//! This module provides a comprehensive completion system that supports:
//! - Multiple completion providers (file, command, code, history)
//! - Real-time completion suggestions with fuzzy matching
//! - Intelligent ranking and filtering of suggestions
//! - Caching for performance optimization
//! - Keyboard navigation and preview capabilities

mod completion_engine;
mod providers;
mod cache;
mod fuzzy;
mod file_provider;
mod command_provider;
mod code_provider;
mod history_provider;
mod completion_list;
mod completion_input;
mod preview;

pub use completion_engine::*;
pub use providers::*;
pub use cache::*;
pub use fuzzy::*;
pub use file_provider::*;
pub use command_provider::*;
pub use code_provider::*;
pub use history_provider::*;
pub use completion_list::*;
pub use completion_input::*;
pub use preview::*;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Maximum number of completion items to display
pub const MAX_COMPLETIONS: usize = 10;

/// Maximum completion popup height
pub const MAX_POPUP_HEIGHT: u16 = 10;

/// A completion item with title, value, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display title of the completion
    pub title: String,
    
    /// Value to insert when selected
    pub value: String,
    
    /// Additional context or description
    pub description: Option<String>,
    
    /// Source provider that generated this completion
    pub provider: String,
    
    /// Relevance score (higher = more relevant)
    pub score: f64,
    
    /// Optional metadata for the completion
    pub metadata: Option<serde_json::Value>,
}

impl CompletionItem {
    /// Create a new completion item
    pub fn new(title: impl Into<String>, value: impl Into<String>, provider: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            description: None,
            provider: provider.into(),
            score: 1.0,
            metadata: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set score
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

impl fmt::Display for CompletionItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title)
    }
}

/// Completion request context
#[derive(Debug, Clone)]
pub struct CompletionContext {
    /// Current input text
    pub text: String,
    
    /// Cursor position in the text
    pub cursor_pos: usize,
    
    /// Working directory for file completions
    pub working_dir: Option<String>,
    
    /// Current command context (if in command mode)
    pub command_context: Option<String>,
    
    /// Language context for code completions
    pub language: Option<String>,
    
    /// Maximum number of completions to return
    pub max_results: usize,
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            working_dir: None,
            command_context: None,
            language: None,
            max_results: MAX_COMPLETIONS,
        }
    }
}

impl CompletionContext {
    /// Create a new completion context
    pub fn new(text: impl Into<String>, cursor_pos: usize) -> Self {
        Self {
            text: text.into(),
            cursor_pos,
            ..Default::default()
        }
    }

    /// Get the current word at cursor position
    pub fn current_word(&self) -> &str {
        let text = &self.text[..self.cursor_pos];
        let start = text.rfind(|c: char| c.is_whitespace() || c == '/' || c == '\\')
            .map(|i| i + 1)
            .unwrap_or(0);
        &self.text[start..self.cursor_pos]
    }

    /// Get text before the current word
    pub fn prefix(&self) -> &str {
        let word_start = self.text[..self.cursor_pos].rfind(|c: char| c.is_whitespace() || c == '/' || c == '\\')
            .map(|i| i + 1)
            .unwrap_or(0);
        &self.text[..word_start]
    }

    /// Get text after cursor
    pub fn suffix(&self) -> &str {
        &self.text[self.cursor_pos..]
    }

    /// Check if we're completing a file path
    pub fn is_file_path(&self) -> bool {
        let current = self.current_word();
        current.contains('/') || current.contains('\\') || current.starts_with('.')
    }

    /// Check if we're completing a command
    pub fn is_command(&self) -> bool {
        self.prefix().trim().is_empty() || self.command_context.is_some()
    }
}

/// Events emitted by the completion system
#[derive(Debug, Clone)]
pub enum CompletionEvent {
    /// Completions opened with items at position
    Opened {
        items: Vec<CompletionItem>,
        x: u16,
        y: u16,
    },
    
    /// Completions filtered with new query
    Filtered {
        query: String,
        items: Vec<CompletionItem>,
    },
    
    /// Completion item selected
    Selected {
        item: CompletionItem,
        insert: bool,
    },
    
    /// Completions closed
    Closed,
    
    /// Completion position changed
    Repositioned {
        x: u16,
        y: u16,
    },
}

/// Messages for controlling the completion system
#[derive(Debug, Clone)]
pub enum CompletionMessage {
    /// Request completions for the given context
    Request(CompletionContext),
    
    /// Filter current completions with query
    Filter {
        query: String,
        reopen: bool,
        x: u16,
        y: u16,
    },
    
    /// Reposition completion popup
    Reposition {
        x: u16,
        y: u16,
    },
    
    /// Select completion item
    Select {
        item: CompletionItem,
        insert: bool,
    },
    
    /// Close completions
    Close,
}