//! AI agent abstraction for handling conversations

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{
    llm::{LlmProvider, ChatRequest, ProviderResponse, Message, MessageRole},
    app::AppEvent,
};

/// An AI agent that manages conversations with an LLM provider
pub struct Agent {
    provider: Arc<dyn LlmProvider>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    session_id: String,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        event_tx: mpsc::UnboundedSender<AppEvent>,
        session_id: String,
    ) -> Self {
        Self {
            provider,
            event_tx,
            session_id,
        }
    }
    
    /// Send a message to the agent and get a response
    pub async fn send_message(
        &self,
        messages: Vec<Message>,
        system_message: Option<String>,
    ) -> Result<ProviderResponse> {
        debug!("Agent sending message to provider: {}", self.provider.name());
        
        let request = ChatRequest {
            messages,
            tools: Vec::new(), // TODO: Load tools from config
            system_message,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: false,
            metadata: std::collections::HashMap::new(),
        };
        
        match self.provider.chat_completion(request).await {
            Ok(response) => {
                info!(
                    "Agent received response from provider: {} tokens",
                    response.usage.total_tokens
                );
                
                // Send event
                let _ = self.event_tx.send(AppEvent::MessageReceived {
                    session_id: self.session_id.clone(),
                    message_id: uuid::Uuid::new_v4().to_string(),
                });
                
                Ok(response)
            }
            Err(e) => {
                error!("Agent error: {}", e);
                
                // Send error event
                let _ = self.event_tx.send(AppEvent::Error {
                    error: e.to_string(),
                });
                
                Err(e.into())
            }
        }
    }
    
    /// Send a message and stream the response
    pub async fn send_message_stream(
        &self,
        messages: Vec<Message>,
        system_message: Option<String>,
    ) -> Result<mpsc::UnboundedReceiver<String>> {
        debug!("Agent sending streaming message to provider: {}", self.provider.name());
        
        let request = ChatRequest {
            messages,
            tools: Vec::new(), // TODO: Load tools from config
            system_message,
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: true,
            metadata: std::collections::HashMap::new(),
        };
        
        let (tx, rx) = mpsc::unbounded_channel();
        let provider = self.provider.clone();
        let event_tx = self.event_tx.clone();
        let session_id = self.session_id.clone();
        let message_id = uuid::Uuid::new_v4().to_string();
        
        tokio::spawn(async move {
            match provider.chat_completion_stream(request).await {
                Ok(mut stream) => {
                    // Send stream started event
                    let _ = event_tx.send(AppEvent::StreamStarted {
                        session_id: session_id.clone(),
                        message_id: message_id.clone(),
                    });
                    
                    use futures::StreamExt;
                    while let Some(event_result) = stream.next().await {
                        match event_result {
                            Ok(event) => {
                                match event {
                                    crate::llm::ProviderEvent::ContentDelta { delta } => {
                                        if tx.send(delta.clone()).is_err() {
                                            break; // Receiver dropped
                                        }
                                        
                                        let _ = event_tx.send(AppEvent::StreamChunk {
                                            session_id: session_id.clone(),
                                            message_id: message_id.clone(),
                                            chunk: delta,
                                        });
                                    }
                                    crate::llm::ProviderEvent::ContentStop => {
                                        break;
                                    }
                                    _ => {} // Handle other events as needed
                                }
                            }
                            Err(e) => {
                                error!("Stream error: {}", e);
                                let _ = event_tx.send(AppEvent::Error {
                                    error: e.to_string(),
                                });
                                break;
                            }
                        }
                    }
                    
                    // Send stream ended event
                    let _ = event_tx.send(AppEvent::StreamEnded {
                        session_id,
                        message_id,
                    });
                }
                Err(e) => {
                    error!("Agent streaming error: {}", e);
                    let _ = event_tx.send(AppEvent::Error {
                        error: e.to_string(),
                    });
                }
            }
        });
        
        Ok(rx)
    }
    
    /// Get the provider name
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
    
    /// Get the model name
    pub fn model_name(&self) -> &str {
        self.provider.model()
    }
}