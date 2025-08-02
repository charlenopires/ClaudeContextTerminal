//! Conversation management and message handling

use anyhow::Result;
use std::{sync::Arc, collections::HashMap};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, error};

use crate::{
    llm::{LlmProvider, Message, MessageRole, ProviderResponse},
    app::Agent,
    session::SessionManager,
};

/// A conversation instance that manages messages and AI interactions
pub struct Conversation {
    pub session_id: String,
    messages: Arc<RwLock<Vec<Message>>>,
    agent: Agent,
    session_manager: Arc<SessionManager>,
    system_message: Option<String>,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(
        session_id: String,
        agent: Agent,
        session_manager: Arc<SessionManager>,
        system_message: Option<String>,
    ) -> Self {
        Self {
            session_id,
            messages: Arc::new(RwLock::new(Vec::new())),
            agent,
            session_manager,
            system_message,
        }
    }
    
    /// Load existing messages from the session
    pub async fn load_messages(&self) -> Result<()> {
        let messages = self.session_manager.get_messages(&self.session_id, None).await?;
        *self.messages.write().await = messages;
        Ok(())
    }
    
    /// Send a message and get a response
    pub async fn send_message(&self, content: String) -> Result<ProviderResponse> {
        debug!("Sending message in conversation: {}", self.session_id);
        
        // Create user message
        let user_message = Message::new_user(content);
        
        // Add to conversation
        self.add_message(user_message.clone()).await?;
        
        // Get current messages for context
        let messages = self.messages.read().await.clone();
        
        // Send to agent
        let response = self.agent.send_message(messages, self.system_message.clone()).await?;
        
        // Create assistant message
        let assistant_message = Message::new_assistant(response.content.clone());
        
        // Add response to conversation
        self.add_message(assistant_message).await?;
        
        // Update session usage
        self.session_manager.update_session_usage(
            &self.session_id,
            &response.usage,
            0.0, // TODO: Calculate cost
        ).await?;
        
        info!(
            "Conversation {} - Message exchange completed. Tokens: {}",
            self.session_id, response.usage.total_tokens
        );
        
        Ok(response)
    }
    
    /// Send a message and stream the response
    pub async fn send_message_stream(&self, content: String) -> Result<mpsc::UnboundedReceiver<String>> {
        debug!("Sending streaming message in conversation: {}", self.session_id);
        
        // Create user message
        let user_message = Message::new_user(content);
        
        // Add to conversation
        self.add_message(user_message.clone()).await?;
        
        // Get current messages for context
        let messages = self.messages.read().await.clone();
        
        // Send to agent for streaming
        let stream_rx = self.agent.send_message_stream(messages, self.system_message.clone()).await?;
        
        Ok(stream_rx)
    }
    
    /// Add a message to the conversation
    pub async fn add_message(&self, message: Message) -> Result<()> {
        // Add to in-memory conversation
        self.messages.write().await.push(message.clone());
        
        // Persist to database
        self.session_manager.add_message(&self.session_id, &message).await?;
        
        Ok(())
    }
    
    /// Get all messages in the conversation
    pub async fn get_messages(&self) -> Vec<Message> {
        self.messages.read().await.clone()
    }
    
    /// Get the last N messages
    pub async fn get_recent_messages(&self, count: usize) -> Vec<Message> {
        let messages = self.messages.read().await;
        messages.iter().rev().take(count).rev().cloned().collect()
    }
    
    /// Clear the conversation (keep in database but clear memory)
    pub async fn clear(&self) {
        self.messages.write().await.clear();
    }
    
    /// Get conversation statistics
    pub async fn get_stats(&self) -> ConversationStats {
        let messages = self.messages.read().await;
        let message_count = messages.len();
        let user_messages = messages.iter().filter(|m| m.role == MessageRole::User).count();
        let assistant_messages = messages.iter().filter(|m| m.role == MessageRole::Assistant).count();
        
        ConversationStats {
            session_id: self.session_id.clone(),
            total_messages: message_count,
            user_messages,
            assistant_messages,
            last_activity: messages.last().map(|m| m.timestamp),
        }
    }
    
    /// Set the system message for this conversation
    pub fn set_system_message(&mut self, system_message: Option<String>) {
        self.system_message = system_message;
    }
    
    /// Get the system message
    pub fn get_system_message(&self) -> Option<&String> {
        self.system_message.as_ref()
    }
}

/// Conversation statistics
#[derive(Debug, Clone)]
pub struct ConversationStats {
    pub session_id: String,
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Conversation manager for handling multiple conversations
pub struct ConversationManager {
    conversations: Arc<RwLock<HashMap<String, Arc<Conversation>>>>,
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new() -> Self {
        Self {
            conversations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Start a new conversation
    pub async fn start_conversation(
        &self,
        session_id: String,
        llm_provider: Arc<dyn LlmProvider>,
    ) -> Result<Arc<Conversation>> {
        // Create event channel for the agent
        let (event_tx, _event_rx) = mpsc::unbounded_channel();
        
        // Create agent
        let agent = Agent::new(llm_provider, event_tx, session_id.clone());
        
        // Create session manager (this should be passed in, but for now create a new one)
        // TODO: Pass session manager from app
        let session_manager = Arc::new(
            crate::session::SessionManager::new(std::path::Path::new("./data")).await?
        );
        
        // Create conversation
        let conversation = Arc::new(Conversation::new(
            session_id.clone(),
            agent,
            session_manager,
            None, // TODO: Load system message from config
        ));
        
        // Load existing messages
        conversation.load_messages().await?;
        
        // Store conversation
        self.conversations.write().await.insert(session_id, conversation.clone());
        
        Ok(conversation)
    }
    
    /// Get an existing conversation
    pub async fn get_conversation(&self, session_id: &str) -> Option<Arc<Conversation>> {
        self.conversations.read().await.get(session_id).cloned()
    }
    
    /// End a conversation
    pub async fn end_conversation(&self, session_id: &str) -> Result<()> {
        if let Some(conversation) = self.conversations.write().await.remove(session_id) {
            conversation.clear().await;
            info!("Conversation ended: {}", session_id);
        }
        
        Ok(())
    }
    
    /// List active conversations
    pub async fn list_conversations(&self) -> Vec<String> {
        self.conversations.read().await.keys().cloned().collect()
    }
    
    /// Get conversation statistics for all active conversations
    pub async fn get_all_stats(&self) -> Result<Vec<ConversationStats>> {
        let conversations = self.conversations.read().await;
        let mut stats = Vec::new();
        
        for conversation in conversations.values() {
            stats.push(conversation.get_stats().await);
        }
        
        Ok(stats)
    }
    
    /// Clear all conversations
    pub async fn clear_all(&self) -> Result<()> {
        let conversation_ids: Vec<String> = self.conversations.read().await.keys().cloned().collect();
        
        for session_id in conversation_ids {
            self.end_conversation(&session_id).await?;
        }
        
        Ok(())
    }
}