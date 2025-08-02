//! Application events for the event-driven architecture

use serde::{Deserialize, Serialize};

/// Events that can occur in the application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AppEvent {
    /// A new session was created
    SessionCreated {
        session_id: String,
    },
    
    /// A session was updated
    SessionUpdated {
        session_id: String,
    },
    
    /// A session was deleted
    SessionDeleted {
        session_id: String,
    },
    
    /// A message was sent by the user
    MessageSent {
        session_id: String,
        message_id: String,
    },
    
    /// A message was received from the AI
    MessageReceived {
        session_id: String,
        message_id: String,
    },
    
    /// A conversation was started
    ConversationStarted {
        session_id: String,
    },
    
    /// A conversation was ended
    ConversationEnded {
        session_id: String,
    },
    
    /// A streaming response started
    StreamStarted {
        session_id: String,
        message_id: String,
    },
    
    /// A streaming response chunk was received
    StreamChunk {
        session_id: String,
        message_id: String,
        chunk: String,
    },
    
    /// A streaming response ended
    StreamEnded {
        session_id: String,
        message_id: String,
    },
    
    /// A tool was called
    ToolCalled {
        session_id: String,
        tool_name: String,
        tool_id: String,
    },
    
    /// A tool call completed
    ToolCompleted {
        session_id: String,
        tool_id: String,
        result: String,
    },
    
    /// An error occurred
    Error {
        error: String,
    },
    
    /// Application is shutting down
    Shutdown,
}

impl AppEvent {
    /// Get the session ID associated with this event, if any
    pub fn session_id(&self) -> Option<&str> {
        match self {
            AppEvent::SessionCreated { session_id }
            | AppEvent::SessionUpdated { session_id }
            | AppEvent::SessionDeleted { session_id }
            | AppEvent::MessageSent { session_id, .. }
            | AppEvent::MessageReceived { session_id, .. }
            | AppEvent::ConversationStarted { session_id }
            | AppEvent::ConversationEnded { session_id }
            | AppEvent::StreamStarted { session_id, .. }
            | AppEvent::StreamChunk { session_id, .. }
            | AppEvent::StreamEnded { session_id, .. }
            | AppEvent::ToolCalled { session_id, .. }
            | AppEvent::ToolCompleted { session_id, .. } => Some(session_id),
            AppEvent::Error { .. } | AppEvent::Shutdown => None,
        }
    }
    
    /// Check if this event is related to streaming
    pub fn is_streaming_event(&self) -> bool {
        matches!(
            self,
            AppEvent::StreamStarted { .. }
                | AppEvent::StreamChunk { .. }
                | AppEvent::StreamEnded { .. }
        )
    }
    
    /// Check if this event is an error
    pub fn is_error(&self) -> bool {
        matches!(self, AppEvent::Error { .. })
    }
}