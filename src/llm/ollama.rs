use std::{collections::HashMap, pin::Pin};
use async_trait::async_trait;
use futures::{Stream, StreamExt, stream};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::{
    provider::LlmProvider,
    types::{ChatRequest, ProviderResponse, ProviderEvent, ProviderConfig, Message, ContentBlock, MessageRole, TokenUsage, FinishReason},
    errors::{LlmError, LlmResult},
};

/// Ollama API configuration
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    default_model: String,
}

/// Ollama chat request format
#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

/// Ollama chat completion request
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

/// Ollama generate request (for single prompts)
#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

/// Ollama response format for chat
#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_duration: Option<u64>,
}

/// Ollama response format for generate
#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
struct OllamaModelsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    modified_at: String,
    size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
}

impl OllamaProvider {
    /// Create a new Ollama provider from configuration
    pub fn new(config: ProviderConfig) -> LlmResult<Self> {
        let client = Client::new();
        let base_url = config.base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
        
        Ok(Self {
            client,
            base_url,
            default_model: config.model,
        })
    }

    /// Get available models from Ollama
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        let url = format!("{}/api/tags", self.base_url);
        
        debug!("Fetching Ollama models from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e))?;

        if !response.status().is_success() {
            return Err(LlmError::ApiError(format!(
                "Failed to fetch models: {}",
                response.status()
            )));
        }

        let models_response: OllamaModelsResponse = response
            .json()
            .await
            .map_err(|e| LlmError::HttpError(e))?;

        let model_names = models_response
            .models
            .into_iter()
            .map(|m| m.name)
            .collect();

        debug!("Available Ollama models: {:?}", model_names);
        Ok(model_names)
    }

    /// Check if Ollama server is running
    pub async fn health_check(&self) -> Result<bool, LlmError> {
        let url = format!("{}/api/tags", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Convert our Message format to Ollama's format
    fn convert_messages(messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| OllamaMessage {
                role: match msg.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(), 
                    MessageRole::System => "system".to_string(),
                    MessageRole::Tool => "user".to_string(), // Ollama doesn't have tool role
                },
                content: msg.get_text_content().unwrap_or_default(),
            })
            .collect()
    }

    /// Parse streaming response
    fn parse_stream_chunk(line: &str) -> Option<String> {
        if line.trim().is_empty() {
            return None;
        }

        // Try parsing as chat response first
        if let Ok(chat_response) = serde_json::from_str::<OllamaChatResponse>(line) {
            if !chat_response.message.content.is_empty() {
                return Some(chat_response.message.content);
            }
        }

        // Try parsing as generate response
        if let Ok(gen_response) = serde_json::from_str::<OllamaGenerateResponse>(line) {
            if !gen_response.response.is_empty() {
                return Some(gen_response.response);
            }
        }

        None
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ProviderResponse> {
        let url = format!("{}/api/chat", self.base_url);

        debug!("Sending Ollama chat request to: {}", url);

        let ollama_request = OllamaChatRequest {
            model: self.default_model.clone(),
            messages: Self::convert_messages(&request.messages),
            stream: false,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            format: None, // Could be made configurable
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(LlmError::ApiError(format!(
                "Ollama API error {}: {}",
                status, error_text
            )));
        }

        let ollama_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| LlmError::HttpError(e))?;

        let mut metadata = HashMap::new();
        
        // Add performance metrics if available
        if let Some(total_duration) = ollama_response.total_duration {
            metadata.insert("total_duration_ns".to_string(), serde_json::Value::Number(serde_json::Number::from(total_duration)));
        }
        if let Some(eval_count) = ollama_response.eval_count {
            metadata.insert("eval_count".to_string(), serde_json::Value::Number(serde_json::Number::from(eval_count)));
        }
        if let Some(prompt_eval_count) = ollama_response.prompt_eval_count {
            metadata.insert("prompt_eval_count".to_string(), serde_json::Value::Number(serde_json::Number::from(prompt_eval_count)));
        }

        metadata.insert("model".to_string(), serde_json::Value::String(self.default_model.clone()));
        metadata.insert("provider".to_string(), serde_json::Value::String("ollama".to_string()));

        // Create usage information
        let usage = TokenUsage {
            input_tokens: ollama_response.prompt_eval_count.unwrap_or(0),
            output_tokens: ollama_response.eval_count.unwrap_or(0),
            total_tokens: ollama_response.prompt_eval_count.unwrap_or(0) + ollama_response.eval_count.unwrap_or(0),
        };

        Ok(ProviderResponse {
            content: ollama_response.message.content,
            tool_calls: Vec::new(), // Ollama doesn't support function calling yet
            usage,
            finish_reason: if ollama_response.done { Some(FinishReason::Stop) } else { None },
            metadata,
        })
    }

    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = LlmResult<ProviderEvent>> + Send>>> {
        let url = format!("{}/api/chat", self.base_url);

        debug!("Starting Ollama streaming chat request to: {}", url);

        let ollama_request = OllamaChatRequest {
            model: self.default_model.clone(),
            messages: Self::convert_messages(&request.messages),
            stream: true,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            format: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await
            .map_err(|e| LlmError::HttpError(e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            return Err(LlmError::ApiError(format!(
                "Ollama streaming API error {}: {}",
                status, error_text
            )));
        }

        let stream = response
            .bytes_stream()
            .map(|result| {
                result.map_err(|e| LlmError::HttpError(e))
            })
            .flat_map(|chunk_result| {
                futures::stream::iter(match chunk_result {
                    Ok(chunk) => {
                        let text = String::from_utf8_lossy(&chunk);
                        text.lines()
                            .filter_map(Self::parse_stream_chunk)
                            .map(|content| Ok(ProviderEvent::ContentDelta { delta: content }))
                            .collect::<Vec<_>>()
                    }
                    Err(e) => vec![Err(e)],
                })
            });

        Ok(Box::pin(stream))
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn model(&self) -> &str {
        &self.default_model
    }

    fn validate_config(&self) -> LlmResult<()> {
        // For Ollama, we mainly need to check if the server is accessible
        // This is done asynchronously in health_check, so we'll just return Ok here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stream_chunk() {
        // Test chat response parsing
        let chat_chunk = r#"{"message":{"role":"assistant","content":"Hello"},"done":false}"#;
        assert_eq!(
            OllamaProvider::parse_stream_chunk(chat_chunk),
            Some("Hello".to_string())
        );

        // Test generate response parsing
        let gen_chunk = r#"{"response":"World","done":false}"#;
        assert_eq!(
            OllamaProvider::parse_stream_chunk(gen_chunk),
            Some("World".to_string())
        );

        // Test empty content
        let empty_chunk = r#"{"message":{"role":"assistant","content":""},"done":true}"#;
        assert_eq!(OllamaProvider::parse_stream_chunk(empty_chunk), None);
    }

    #[test]
    fn test_convert_messages() {
        let messages = vec![
            Message::new_user("Hello".to_string()),
            Message::new_assistant("Hi there!".to_string()),
        ];

        let ollama_messages = OllamaProvider::convert_messages(&messages);
        
        assert_eq!(ollama_messages.len(), 2);
        assert_eq!(ollama_messages[0].role, "user");
        assert_eq!(ollama_messages[0].content, "Hello");
        assert_eq!(ollama_messages[1].role, "assistant");
        assert_eq!(ollama_messages[1].content, "Hi there!");
    }

    #[tokio::test]
    async fn test_ollama_provider_creation() {
        let config = ProviderConfig {
            provider_type: "ollama".to_string(),
            model: "llama2".to_string(),
            base_url: None,
            ..Default::default()
        };
        let provider = OllamaProvider::new(config).unwrap();
        assert_eq!(provider.base_url, "http://localhost:11434");
        assert_eq!(provider.default_model, "llama2");
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llama2");
    }

    #[tokio::test]
    async fn test_ollama_provider_custom_url() {
        let config = ProviderConfig {
            provider_type: "ollama".to_string(),
            model: "mistral".to_string(),
            base_url: Some("http://custom-ollama:8080".to_string()),
            ..Default::default()
        };
        let provider = OllamaProvider::new(config).unwrap();
        assert_eq!(provider.base_url, "http://custom-ollama:8080");
        assert_eq!(provider.default_model, "mistral");
    }
}