//! Real-time streaming support for chat messages
//!
//! This module provides real-time streaming capabilities for chat messages,
//! handling incremental updates, typing indicators, and streaming state management.

use super::message_types::{ChatMessage, StreamingState, FinishReason};
use crate::llm::types::{ProviderEvent, MessageRole, ContentBlock};
use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::{broadcast, mpsc},
    time::interval,
};
use uuid::Uuid;

/// Maximum number of concurrent streaming messages
const MAX_CONCURRENT_STREAMS: usize = 10;

/// Streaming update interval in milliseconds
const STREAM_UPDATE_INTERVAL_MS: u64 = 50;

/// Timeout for streaming operations in seconds
const STREAM_TIMEOUT_SECONDS: u64 = 300; // 5 minutes

/// Streaming manager for handling real-time message updates
pub struct StreamingManager {
    active_streams: Arc<Mutex<HashMap<String, StreamingMessage>>>,
    update_sender: broadcast::Sender<StreamingUpdate>,
    _update_receiver: broadcast::Receiver<StreamingUpdate>,
    command_sender: mpsc::UnboundedSender<StreamingCommand>,
    command_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<StreamingCommand>>>>,
}

/// Streaming message state
#[derive(Debug, Clone)]
struct StreamingMessage {
    id: String,
    message: ChatMessage,
    started_at: Instant,
    last_update: Instant,
    buffer: String,
    thinking_buffer: Option<String>,
    error_count: usize,
    max_errors: usize,
}

/// Streaming update events
#[derive(Debug, Clone)]
pub enum StreamingUpdate {
    /// Message started streaming
    StreamStarted {
        message_id: String,
        role: MessageRole,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Content delta received
    ContentDelta {
        message_id: String,
        delta: String,
        accumulated_content: String,
    },
    
    /// Thinking content delta received
    ThinkingDelta {
        message_id: String,
        delta: String,
        accumulated_thinking: String,
    },
    
    /// Tool use started
    ToolUseStarted {
        message_id: String,
        tool_call_id: String,
        tool_name: String,
    },
    
    /// Tool use completed
    ToolUseCompleted {
        message_id: String,
        tool_call_id: String,
        result: String,
    },
    
    /// Streaming completed
    StreamCompleted {
        message_id: String,
        final_message: ChatMessage,
        finish_reason: FinishReason,
    },
    
    /// Streaming failed
    StreamFailed {
        message_id: String,
        error: String,
    },
    
    /// Streaming cancelled
    StreamCancelled {
        message_id: String,
    },
    
    /// Typing indicator update
    TypingIndicator {
        message_id: String,
        is_typing: bool,
        typing_text: Option<String>,
    },
}

/// Commands for controlling streaming
#[derive(Debug)]
pub enum StreamingCommand {
    /// Start a new streaming message
    StartStream {
        message_id: String,
        role: MessageRole,
    },
    
    /// Process a provider event
    ProcessEvent {
        message_id: String,
        event: ProviderEvent,
    },
    
    /// Cancel a streaming message
    CancelStream {
        message_id: String,
    },
    
    /// Cancel all streaming messages
    CancelAllStreams,
    
    /// Update typing indicator
    UpdateTyping {
        message_id: String,
        is_typing: bool,
        text: Option<String>,
    },
    
    /// Clean up expired streams
    CleanupExpiredStreams,
}

/// Streaming subscription handle
pub struct StreamingSubscription {
    receiver: broadcast::Receiver<StreamingUpdate>,
}

/// Animation frames for typing indicators
const TYPING_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
const THINKING_FRAMES: &[&str] = &["🤔", "💭", "🧠", "💡"];

impl StreamingManager {
    /// Create a new streaming manager
    pub fn new() -> Self {
        let (update_sender, update_receiver) = broadcast::channel(1000);
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        
        Self {
            active_streams: Arc::new(Mutex::new(HashMap::new())),
            update_sender,
            _update_receiver: update_receiver,
            command_sender,
            command_receiver: Arc::new(Mutex::new(Some(command_receiver))),
        }
    }

    /// Start the streaming manager background task
    pub async fn start(&self) -> Result<()> {
        let streams = self.active_streams.clone();
        let update_sender = self.update_sender.clone();
        let command_receiver = self.command_receiver.clone();
        
        // Take the receiver out of the Arc<Mutex<>>
        let mut receiver = {
            let mut guard = command_receiver.lock().unwrap();
            guard.take().ok_or_else(|| anyhow::anyhow!("Streaming manager already started"))?
        };

        // Spawn the main processing task
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(30));
            
            loop {
                tokio::select! {
                    // Process commands
                    Some(command) = receiver.recv() => {
                        if let Err(e) = Self::process_command(command, &streams, &update_sender).await {
                            eprintln!("Error processing streaming command: {}", e);
                        }
                    }
                    
                    // Periodic cleanup
                    _ = cleanup_interval.tick() => {
                        Self::cleanup_expired_streams(&streams, &update_sender).await;
                    }
                }
            }
        });

        // Spawn animation update task
        let streams_clone = self.active_streams.clone();
        let update_sender_clone = self.update_sender.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(STREAM_UPDATE_INTERVAL_MS));
            
            loop {
                interval.tick().await;
                Self::update_animations(&streams_clone, &update_sender_clone).await;
            }
        });

        Ok(())
    }

    /// Subscribe to streaming updates
    pub fn subscribe(&self) -> StreamingSubscription {
        StreamingSubscription {
            receiver: self.update_sender.subscribe(),
        }
    }

    /// Start streaming a new message
    pub async fn start_stream(&self, message_id: String, role: MessageRole) -> Result<()> {
        self.command_sender.send(StreamingCommand::StartStream { message_id, role })?;
        Ok(())
    }

    /// Process a provider event
    pub async fn process_event(&self, message_id: String, event: ProviderEvent) -> Result<()> {
        self.command_sender.send(StreamingCommand::ProcessEvent { message_id, event })?;
        Ok(())
    }

    /// Cancel a streaming message
    pub async fn cancel_stream(&self, message_id: String) -> Result<()> {
        self.command_sender.send(StreamingCommand::CancelStream { message_id })?;
        Ok(())
    }

    /// Cancel all streaming messages
    pub async fn cancel_all_streams(&self) -> Result<()> {
        self.command_sender.send(StreamingCommand::CancelAllStreams)?;
        Ok(())
    }

    /// Update typing indicator
    pub async fn update_typing(&self, message_id: String, is_typing: bool, text: Option<String>) -> Result<()> {
        self.command_sender.send(StreamingCommand::UpdateTyping { message_id, is_typing, text })?;
        Ok(())
    }

    /// Get current streaming statistics
    pub fn get_stats(&self) -> StreamingStats {
        let streams = self.active_streams.lock().unwrap();
        StreamingStats {
            active_streams: streams.len(),
            total_messages_processed: 0, // Would need to track this
            average_stream_duration: Duration::from_secs(0), // Would need to track this
        }
    }

    // Internal processing methods

    async fn process_command(
        command: StreamingCommand,
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        match command {
            StreamingCommand::StartStream { message_id, role } => {
                Self::handle_start_stream(message_id, role, streams, update_sender).await
            }
            StreamingCommand::ProcessEvent { message_id, event } => {
                Self::handle_process_event(message_id, event, streams, update_sender).await
            }
            StreamingCommand::CancelStream { message_id } => {
                Self::handle_cancel_stream(message_id, streams, update_sender).await
            }
            StreamingCommand::CancelAllStreams => {
                Self::handle_cancel_all_streams(streams, update_sender).await
            }
            StreamingCommand::UpdateTyping { message_id, is_typing, text } => {
                Self::handle_update_typing(message_id, is_typing, text, update_sender).await
            }
            StreamingCommand::CleanupExpiredStreams => {
                Self::cleanup_expired_streams(streams, update_sender).await;
                Ok(())
            }
        }
    }

    async fn handle_start_stream(
        message_id: String,
        role: MessageRole,
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        let mut streams_guard = streams.lock().unwrap();
        
        // Check if we've hit the concurrent stream limit
        if streams_guard.len() >= MAX_CONCURRENT_STREAMS {
            return Err(anyhow::anyhow!("Maximum concurrent streams reached"));
        }

        // Create new streaming message
        let mut message = ChatMessage::new(role.clone(), Vec::new());
        message.id = message_id.clone();
        message.set_streaming_state(StreamingState::Streaming);

        let streaming_message = StreamingMessage {
            id: message_id.clone(),
            message: message.clone(),
            started_at: Instant::now(),
            last_update: Instant::now(),
            buffer: String::new(),
            thinking_buffer: None,
            error_count: 0,
            max_errors: 5,
        };

        streams_guard.insert(message_id.clone(), streaming_message);
        drop(streams_guard);

        // Send update
        let _ = update_sender.send(StreamingUpdate::StreamStarted {
            message_id,
            role,
            timestamp: message.timestamp,
        });

        Ok(())
    }

    async fn handle_process_event(
        message_id: String,
        event: ProviderEvent,
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        let mut streams_guard = streams.lock().unwrap();
        
        if let Some(stream) = streams_guard.get_mut(&message_id) {
            stream.last_update = Instant::now();
            
            match event {
                ProviderEvent::ContentDelta { delta } => {
                    stream.buffer.push_str(&delta);
                    stream.message.update_content(stream.buffer.clone());
                    
                    let _ = update_sender.send(StreamingUpdate::ContentDelta {
                        message_id: message_id.clone(),
                        delta,
                        accumulated_content: stream.buffer.clone(),
                    });
                }
                
                ProviderEvent::ToolUseStart { tool_call } => {
                    stream.message.add_tool_call(tool_call.clone());
                    
                    let _ = update_sender.send(StreamingUpdate::ToolUseStarted {
                        message_id: message_id.clone(),
                        tool_call_id: tool_call.id,
                        tool_name: tool_call.name,
                    });
                }
                
                ProviderEvent::Done { response } => {
                    // Finalize the message
                    stream.message.set_streaming_state(StreamingState::Complete);
                    if let Some(finish_reason) = response.finish_reason {
                        stream.message.set_finish_reason(match finish_reason {
                            crate::llm::types::FinishReason::Stop => FinishReason::Stop,
                            crate::llm::types::FinishReason::Length => FinishReason::Length,
                            crate::llm::types::FinishReason::ContentFilter => FinishReason::ContentFilter,
                            crate::llm::types::FinishReason::ToolCalls => FinishReason::ToolCalls,
                            crate::llm::types::FinishReason::Error => FinishReason::Error("Provider error".to_string()),
                        });
                    }
                    
                    let final_message = stream.message.clone();
                    let finish_reason = stream.message.finish_reason.clone().unwrap_or(FinishReason::Stop);
                    
                    // Remove from active streams
                    streams_guard.remove(&message_id);
                    drop(streams_guard);
                    
                    let _ = update_sender.send(StreamingUpdate::StreamCompleted {
                        message_id,
                        final_message,
                        finish_reason,
                    });
                }
                
                ProviderEvent::Error { error } => {
                    stream.error_count += 1;
                    
                    if stream.error_count >= stream.max_errors {
                        stream.message.set_streaming_state(StreamingState::Failed(error.clone()));
                        streams_guard.remove(&message_id);
                        drop(streams_guard);
                        
                        let _ = update_sender.send(StreamingUpdate::StreamFailed {
                            message_id,
                            error,
                        });
                    }
                }
                
                _ => {
                    // Handle other event types as needed
                }
            }
        } else {
            return Err(anyhow::anyhow!("Stream not found: {}", message_id));
        }

        Ok(())
    }

    async fn handle_cancel_stream(
        message_id: String,
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        let mut streams_guard = streams.lock().unwrap();
        
        if let Some(mut stream) = streams_guard.remove(&message_id) {
            stream.message.set_streaming_state(StreamingState::Cancelled);
            stream.message.set_finish_reason(FinishReason::Cancelled);
            drop(streams_guard);
            
            let _ = update_sender.send(StreamingUpdate::StreamCancelled { message_id });
        }

        Ok(())
    }

    async fn handle_cancel_all_streams(
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        let mut streams_guard = streams.lock().unwrap();
        let message_ids: Vec<String> = streams_guard.keys().cloned().collect();
        streams_guard.clear();
        drop(streams_guard);

        for message_id in message_ids {
            let _ = update_sender.send(StreamingUpdate::StreamCancelled { message_id });
        }

        Ok(())
    }

    async fn handle_update_typing(
        message_id: String,
        is_typing: bool,
        text: Option<String>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) -> Result<()> {
        let _ = update_sender.send(StreamingUpdate::TypingIndicator {
            message_id,
            is_typing,
            typing_text: text,
        });

        Ok(())
    }

    async fn cleanup_expired_streams(
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) {
        let mut streams_guard = streams.lock().unwrap();
        let now = Instant::now();
        let timeout = Duration::from_secs(STREAM_TIMEOUT_SECONDS);
        
        let expired_ids: Vec<String> = streams_guard
            .iter()
            .filter(|(_, stream)| now.duration_since(stream.last_update) > timeout)
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired_ids {
            if let Some(mut stream) = streams_guard.remove(&id) {
                stream.message.set_streaming_state(StreamingState::Failed("Stream timeout".to_string()));
                
                let _ = update_sender.send(StreamingUpdate::StreamFailed {
                    message_id: id,
                    error: "Stream timeout".to_string(),
                });
            }
        }
    }

    async fn update_animations(
        streams: &Arc<Mutex<HashMap<String, StreamingMessage>>>,
        update_sender: &broadcast::Sender<StreamingUpdate>,
    ) {
        let streams_guard = streams.lock().unwrap();
        
        for (message_id, stream) in streams_guard.iter() {
            if stream.message.is_streaming() {
                // Send typing indicator updates for animation
                let frame_index = (stream.started_at.elapsed().as_millis() / 200) as usize;
                let typing_frame = TYPING_FRAMES[frame_index % TYPING_FRAMES.len()];
                
                let _ = update_sender.send(StreamingUpdate::TypingIndicator {
                    message_id: message_id.clone(),
                    is_typing: true,
                    typing_text: Some(typing_frame.to_string()),
                });
            }
        }
    }
}

impl StreamingSubscription {
    /// Receive the next streaming update
    pub async fn recv(&mut self) -> Result<StreamingUpdate> {
        self.receiver.recv().await.map_err(|e| anyhow::anyhow!("Streaming channel error: {}", e))
    }

    /// Try to receive a streaming update without blocking
    pub fn try_recv(&mut self) -> Result<Option<StreamingUpdate>> {
        match self.receiver.try_recv() {
            Ok(update) => Ok(Some(update)),
            Err(broadcast::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Streaming channel error: {}", e)),
        }
    }
}

/// Streaming statistics
#[derive(Debug, Clone)]
pub struct StreamingStats {
    pub active_streams: usize,
    pub total_messages_processed: usize,
    pub average_stream_duration: Duration,
}

/// Streaming message buffer for efficient updates
pub struct StreamingBuffer {
    message_id: String,
    content_buffer: String,
    thinking_buffer: Option<String>,
    last_flush: Instant,
    flush_interval: Duration,
    max_buffer_size: usize,
}

impl StreamingBuffer {
    /// Create a new streaming buffer
    pub fn new(message_id: String) -> Self {
        Self {
            message_id,
            content_buffer: String::new(),
            thinking_buffer: None,
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(100), // Flush every 100ms
            max_buffer_size: 1024, // 1KB buffer
        }
    }

    /// Add content to the buffer
    pub fn add_content(&mut self, delta: &str) -> bool {
        self.content_buffer.push_str(delta);
        self.should_flush()
    }

    /// Add thinking content to the buffer
    pub fn add_thinking(&mut self, delta: &str) -> bool {
        if let Some(ref mut buffer) = self.thinking_buffer {
            buffer.push_str(delta);
        } else {
            self.thinking_buffer = Some(delta.to_string());
        }
        self.should_flush()
    }

    /// Check if buffer should be flushed
    pub fn should_flush(&self) -> bool {
        self.last_flush.elapsed() >= self.flush_interval ||
        self.content_buffer.len() >= self.max_buffer_size ||
        self.thinking_buffer.as_ref().map_or(false, |b| b.len() >= self.max_buffer_size)
    }

    /// Flush the buffer and return content
    pub fn flush(&mut self) -> (String, Option<String>) {
        let content = std::mem::take(&mut self.content_buffer);
        let thinking = self.thinking_buffer.take();
        self.last_flush = Instant::now();
        (content, thinking)
    }

    /// Get current buffer contents without flushing
    pub fn peek(&self) -> (&str, Option<&str>) {
        (&self.content_buffer, self.thinking_buffer.as_deref())
    }
}

/// Typing indicator for showing user input state
pub struct TypingIndicator {
    is_active: bool,
    start_time: Option<Instant>,
    last_frame_time: Instant,
    frame_index: usize,
    custom_text: Option<String>,
}

impl TypingIndicator {
    /// Create a new typing indicator
    pub fn new() -> Self {
        Self {
            is_active: false,
            start_time: None,
            last_frame_time: Instant::now(),
            frame_index: 0,
            custom_text: None,
        }
    }

    /// Start the typing indicator
    pub fn start(&mut self, custom_text: Option<String>) {
        self.is_active = true;
        self.start_time = Some(Instant::now());
        self.custom_text = custom_text;
        self.frame_index = 0;
    }

    /// Stop the typing indicator
    pub fn stop(&mut self) {
        self.is_active = false;
        self.start_time = None;
        self.custom_text = None;
        self.frame_index = 0;
    }

    /// Update animation frame
    pub fn update(&mut self) {
        if self.is_active && self.last_frame_time.elapsed() >= Duration::from_millis(200) {
            self.frame_index = (self.frame_index + 1) % TYPING_FRAMES.len();
            self.last_frame_time = Instant::now();
        }
    }

    /// Get current display text
    pub fn get_text(&self) -> Option<String> {
        if self.is_active {
            if let Some(ref custom) = self.custom_text {
                Some(format!("{} {}", TYPING_FRAMES[self.frame_index], custom))
            } else {
                Some(format!("{} Typing...", TYPING_FRAMES[self.frame_index]))
            }
        } else {
            None
        }
    }

    /// Check if typing indicator is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

impl Default for StreamingManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_streaming_manager_creation() {
        let manager = StreamingManager::new();
        assert!(manager.start().await.is_ok());
    }

    #[tokio::test]
    async fn test_streaming_buffer() {
        let mut buffer = StreamingBuffer::new("test_id".to_string());
        
        assert!(!buffer.add_content("Hello"));
        assert!(!buffer.add_content(" "));
        assert!(!buffer.add_content("World"));
        
        // Should flush after enough content
        let large_content = "x".repeat(2000);
        assert!(buffer.add_content(&large_content));
        
        let (content, thinking) = buffer.flush();
        assert!(content.contains("Hello World"));
        assert!(thinking.is_none());
    }

    #[tokio::test]
    async fn test_typing_indicator() {
        let mut indicator = TypingIndicator::new();
        
        assert!(!indicator.is_active());
        assert!(indicator.get_text().is_none());
        
        indicator.start(Some("Custom message".to_string()));
        assert!(indicator.is_active());
        
        let text = indicator.get_text();
        assert!(text.is_some());
        assert!(text.unwrap().contains("Custom message"));
        
        indicator.stop();
        assert!(!indicator.is_active());
    }

    #[tokio::test]
    async fn test_streaming_flow() {
        let manager = StreamingManager::new();
        assert!(manager.start().await.is_ok());
        
        let mut subscription = manager.subscribe();
        let message_id = Uuid::new_v4().to_string();
        
        // Start streaming
        assert!(manager.start_stream(message_id.clone(), MessageRole::Assistant).await.is_ok());
        
        // Should receive stream started event
        if let Ok(update) = subscription.recv().await {
            match update {
                StreamingUpdate::StreamStarted { message_id: recv_id, role, .. } => {
                    assert_eq!(recv_id, message_id);
                    assert_eq!(role, MessageRole::Assistant);
                }
                _ => panic!("Expected StreamStarted event"),
            }
        }
        
        // Process content delta
        let delta_event = ProviderEvent::ContentDelta { delta: "Hello".to_string() };
        assert!(manager.process_event(message_id.clone(), delta_event).await.is_ok());
        
        // Should receive content delta event
        if let Ok(update) = subscription.recv().await {
            match update {
                StreamingUpdate::ContentDelta { message_id: recv_id, delta, .. } => {
                    assert_eq!(recv_id, message_id);
                    assert_eq!(delta, "Hello");
                }
                _ => panic!("Expected ContentDelta event"),
            }
        }
    }
}