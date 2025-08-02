//! Anthropic provider implementation

use async_trait::async_trait;
use std::{pin::Pin, time::Duration, collections::HashMap};
use futures::{Stream, StreamExt, stream};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE}};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::llm::{
    provider::{LlmProvider, ProviderClientOptions, utils},
    types::{
        ChatRequest, ProviderResponse, ProviderEvent, ProviderConfig, Message, MessageRole,
        ContentBlock, ToolCall, TokenUsage, FinishReason, Tool,
    },
    errors::{LlmError, LlmResult},
};

/// Anthropic API provider
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    client: Client,
    config: ProviderConfig,
    options: ProviderClientOptions,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: ProviderConfig) -> LlmResult<Self> {
        let mut headers = HeaderMap::new();
        
        // Set API key
        if let Some(api_key) = &config.api_key {
            let auth_value = HeaderValue::from_str(api_key)
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key: {}", e)))?;
            headers.insert("x-api-key", auth_value);
        } else {
            return Err(LlmError::ConfigError("API key is required".to_string()));
        }
        
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        
        // Add extra headers
        for (key, value) in &config.extra_headers {
            let header_name: reqwest::header::HeaderName = key.parse()
                .map_err(|e| LlmError::ConfigError(format!("Invalid header name '{}': {}", key, e)))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|e| LlmError::ConfigError(format!("Invalid header value for '{}': {}", key, e)))?;
            headers.insert(header_name, header_value);
        }
        
        let options = ProviderClientOptions::default();
        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(options.timeout_seconds))
            .user_agent(&options.user_agent)
            .build()
            .map_err(|e| LlmError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            client,
            config,
            options,
        })
    }
    
    /// Convert messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_message = None;
        let mut converted_messages = Vec::new();
        
        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    if let Some(text) = msg.get_text_content() {
                        system_message = Some(text);
                    }
                }
                MessageRole::User | MessageRole::Assistant => {
                    let role = match msg.role {
                        MessageRole::User => "user".to_string(),
                        MessageRole::Assistant => "assistant".to_string(),
                        _ => unreachable!(),
                    };
                    
                    let content = self.convert_content_blocks(&msg.content);
                    
                    converted_messages.push(AnthropicMessage {
                        role,
                        content,
                    });
                }
                MessageRole::Tool => {
                    // Tool results are typically merged into user messages
                    if let Some(last_msg) = converted_messages.last_mut() {
                        if last_msg.role == "user" {
                            let tool_content = self.convert_content_blocks(&msg.content);
                            last_msg.content.extend(tool_content);
                        }
                    }
                }
            }
        }
        
        (system_message, converted_messages)
    }
    
    /// Convert content blocks to Anthropic format
    fn convert_content_blocks(&self, blocks: &[ContentBlock]) -> Vec<AnthropicContentBlock> {
        blocks.iter().filter_map(|block| {
            match block {
                ContentBlock::Text { text } => Some(AnthropicContentBlock {
                    block_type: "text".to_string(),
                    text: Some(text.clone()),
                    source: None,
                    tool_use_id: None,
                    name: None,
                    input: None,
                    content: None,
                }),
                ContentBlock::Image { image } => Some(AnthropicContentBlock {
                    block_type: "image".to_string(),
                    text: None,
                    source: Some(AnthropicImageSource {
                        source_type: "base64".to_string(),
                        media_type: image.media_type.clone(),
                        data: image.data.clone(),
                    }),
                    tool_use_id: None,
                    name: None,
                    input: None,
                    content: None,
                }),
                ContentBlock::ToolUse { id, name, input } => Some(AnthropicContentBlock {
                    block_type: "tool_use".to_string(),
                    text: None,
                    source: None,
                    tool_use_id: Some(id.clone()),
                    name: Some(name.clone()),
                    input: Some(input.clone()),
                    content: None,
                }),
                ContentBlock::ToolResult { tool_call_id, content } => Some(AnthropicContentBlock {
                    block_type: "tool_result".to_string(),
                    text: None,
                    source: None,
                    tool_use_id: Some(tool_call_id.clone()),
                    name: None,
                    input: None,
                    content: Some(content.clone()),
                }),
            }
        }).collect()
    }
    
    /// Convert tools to Anthropic format
    fn convert_tools(&self, tools: &[Tool]) -> Vec<AnthropicTool> {
        tools.iter().map(|tool| {
            AnthropicTool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.input_schema.clone(),
            }
        }).collect()
    }
    
    /// Get the API endpoint URL
    fn get_endpoint(&self) -> String {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.anthropic.com");
        format!("{}/v1/messages", base_url)
    }
    
    /// Execute request with retries
    async fn execute_request<T>(&self, request_body: serde_json::Value) -> LlmResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.options.max_retries {
            if attempt > 0 {
                utils::exponential_backoff_with_jitter(attempt, self.options.retry_delay_ms).await;
            }
            
            let response = self.client
                .post(&self.get_endpoint())
                .json(&request_body)
                .send()
                .await;
            
            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<T>().await {
                            Ok(result) => return Ok(result),
                            Err(e) => {
                                last_error = Some(LlmError::HttpError(e));
                                continue;
                            }
                        }
                    } else {
                        let status = resp.status();
                        let error_msg = utils::extract_error_message(resp).await;
                        
                        let error = match status.as_u16() {
                            429 => LlmError::RateLimitError(error_msg),
                            401 | 403 => LlmError::AuthError(error_msg),
                            400 => {
                                if error_msg.contains("context_length_exceeded") || error_msg.contains("too long") {
                                    LlmError::ContextLimitError(error_msg)
                                } else {
                                    LlmError::ApiError(error_msg)
                                }
                            }
                            _ => LlmError::ApiError(error_msg),
                        };
                        
                        if !utils::is_retryable_error(&error) || attempt == self.options.max_retries {
                            return Err(error);
                        }
                        
                        last_error = Some(error);
                    }
                }
                Err(e) => {
                    let error = LlmError::HttpError(e);
                    if !utils::is_retryable_error(&error) || attempt == self.options.max_retries {
                        return Err(error);
                    }
                    last_error = Some(error);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| LlmError::ApiError("Unknown error".to_string())))
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ProviderResponse> {
        let (system_message, messages) = self.convert_messages(&request.messages);
        
        let mut request_body = json!({
            "model": self.config.model,
            "messages": messages,
        });
        
        // Add system message if present
        if let Some(system) = system_message.or(request.system_message) {
            request_body["system"] = json!(system);
        }
        
        // Add optional parameters
        if let Some(max_tokens) = request.max_tokens.or(self.config.max_tokens) {
            request_body["max_tokens"] = json!(max_tokens);
        } else {
            // Anthropic requires max_tokens
            request_body["max_tokens"] = json!(4096);
        }
        
        if let Some(temperature) = request.temperature.or(self.config.temperature) {
            request_body["temperature"] = json!(temperature);
        }
        
        if let Some(top_p) = request.top_p.or(self.config.top_p) {
            request_body["top_p"] = json!(top_p);
        }
        
        if !request.tools.is_empty() {
            request_body["tools"] = json!(self.convert_tools(&request.tools));
        }
        
        // Add extra body parameters
        for (key, value) in &self.config.extra_body {
            request_body[key] = value.clone();
        }
        
        let response: AnthropicResponse = self.execute_request(request_body).await?;
        
        let mut content = String::new();
        let mut tool_calls = Vec::new();
        
        for content_block in response.content {
            match content_block.block_type.as_str() {
                "text" => {
                    if let Some(text) = content_block.text {
                        content.push_str(&text);
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) = (
                        content_block.tool_use_id,
                        content_block.name,
                        content_block.input,
                    ) {
                        tool_calls.push(ToolCall { id, name, arguments: input });
                    }
                }
                _ => {}
            }
        }
        
        let finish_reason = match response.stop_reason.as_deref() {
            Some("end_turn") => Some(FinishReason::Stop),
            Some("max_tokens") => Some(FinishReason::Length),
            Some("tool_use") => Some(FinishReason::ToolCalls),
            _ => None,
        };
        
        Ok(ProviderResponse {
            content,
            tool_calls,
            usage: TokenUsage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
                total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            },
            finish_reason,
            metadata: HashMap::new(),
        })
    }
    
    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = LlmResult<ProviderEvent>> + Send>>> {
        let (system_message, messages) = self.convert_messages(&request.messages);
        
        let mut request_body = json!({
            "model": self.config.model,
            "messages": messages,
            "stream": true,
        });
        
        // Add system message if present
        if let Some(system) = system_message.or(request.system_message) {
            request_body["system"] = json!(system);
        }
        
        // Add optional parameters
        if let Some(max_tokens) = request.max_tokens.or(self.config.max_tokens) {
            request_body["max_tokens"] = json!(max_tokens);
        } else {
            // Anthropic requires max_tokens
            request_body["max_tokens"] = json!(4096);
        }
        
        if let Some(temperature) = request.temperature.or(self.config.temperature) {
            request_body["temperature"] = json!(temperature);
        }
        
        if let Some(top_p) = request.top_p.or(self.config.top_p) {
            request_body["top_p"] = json!(top_p);
        }
        
        if !request.tools.is_empty() {
            request_body["tools"] = json!(self.convert_tools(&request.tools));
        }
        
        // Add extra body parameters
        for (key, value) in &self.config.extra_body {
            request_body[key] = value.clone();
        }
        
        let response = self.client
            .post(&self.get_endpoint())
            .json(&request_body)
            .send()
            .await
            .map_err(LlmError::HttpError)?;
        
        if !response.status().is_success() {
            let error_msg = utils::extract_error_message(response).await;
            return Err(LlmError::ApiError(error_msg));
        }
        
        let stream = response.bytes_stream()
            .map(|result| {
                result.map_err(LlmError::HttpError)
            })
            .filter_map(|chunk_result| async move {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        
                        // Parse SSE format
                        for line in chunk_str.lines() {
                            if line.starts_with("data: ") {
                                let data = &line[6..];
                                
                                match serde_json::from_str::<AnthropicStreamEvent>(data) {
                                    Ok(event) => {
                                        match event.event_type.as_str() {
                                            "content_block_start" => {
                                                return Some(Ok(ProviderEvent::ContentStart));
                                            }
                                            "content_block_delta" => {
                                                if let Some(delta) = event.delta {
                                                    if let Some(text) = delta.text {
                                                        return Some(Ok(ProviderEvent::ContentDelta { delta: text }));
                                                    }
                                                }
                                            }
                                            "content_block_stop" => {
                                                return Some(Ok(ProviderEvent::ContentStop));
                                            }
                                            "message_stop" => {
                                                return Some(Ok(ProviderEvent::ContentStop));
                                            }
                                            _ => {}
                                        }
                                    }
                                    Err(e) => {
                                        return Some(Err(LlmError::JsonError(e)));
                                    }
                                }
                            }
                        }
                        None
                    }
                    Err(e) => Some(Err(e)),
                }
            });
        
        Ok(Box::pin(stream))
    }
    
    fn name(&self) -> &str {
        "anthropic"
    }
    
    fn model(&self) -> &str {
        &self.config.model
    }
    
    fn validate_config(&self) -> LlmResult<()> {
        if self.config.api_key.is_none() {
            return Err(LlmError::ConfigError("API key is required".to_string()));
        }
        
        if self.config.model.is_empty() {
            return Err(LlmError::ConfigError("Model is required".to_string()));
        }
        
        Ok(())
    }
}

// Anthropic API types
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<AnthropicImageSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseContentBlock>,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponseContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta: Option<AnthropicStreamDelta>,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}