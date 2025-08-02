//! OpenAI provider implementation

use async_trait::async_trait;
use std::{pin::Pin, time::Duration, collections::HashMap};
use futures::{Stream, StreamExt, stream};
use reqwest::{Client, header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}};
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

/// OpenAI API provider
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    client: Client,
    config: ProviderConfig,
    options: ProviderClientOptions,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: ProviderConfig) -> LlmResult<Self> {
        let mut headers = HeaderMap::new();
        
        // Set API key
        if let Some(api_key) = &config.api_key {
            let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| LlmError::ConfigError(format!("Invalid API key: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_value);
        } else {
            return Err(LlmError::ConfigError("API key is required".to_string()));
        }
        
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        
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
    
    /// Convert messages to OpenAI format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages.iter().map(|msg| {
            let role = match msg.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
                MessageRole::Tool => "tool".to_string(),
            };
            
            let content = if msg.content.len() == 1 {
                // Single content block - use string format
                match &msg.content[0] {
                    ContentBlock::Text { text } => OpenAIContent::String(text.clone()),
                    _ => OpenAIContent::Array(self.convert_content_blocks(&msg.content)),
                }
            } else {
                // Multiple content blocks - use array format
                OpenAIContent::Array(self.convert_content_blocks(&msg.content))
            };
            
            OpenAIMessage {
                role,
                content,
                tool_calls: None,
                tool_call_id: None,
            }
        }).collect()
    }
    
    /// Convert content blocks to OpenAI format
    fn convert_content_blocks(&self, blocks: &[ContentBlock]) -> Vec<OpenAIContentBlock> {
        blocks.iter().filter_map(|block| {
            match block {
                ContentBlock::Text { text } => Some(OpenAIContentBlock {
                    block_type: "text".to_string(),
                    text: Some(text.clone()),
                    image_url: None,
                }),
                ContentBlock::Image { image } => Some(OpenAIContentBlock {
                    block_type: "image_url".to_string(),
                    text: None,
                    image_url: Some(OpenAIImageUrl {
                        url: format!("data:{};base64,{}", image.media_type, image.data),
                    }),
                }),
                ContentBlock::ToolUse { .. } | ContentBlock::ToolResult { .. } => None,
            }
        }).collect()
    }
    
    /// Convert tools to OpenAI format
    fn convert_tools(&self, tools: &[Tool]) -> Vec<OpenAITool> {
        tools.iter().map(|tool| {
            OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.input_schema.clone(),
                },
            }
        }).collect()
    }
    
    /// Get the API endpoint URL
    fn get_endpoint(&self) -> String {
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.openai.com");
        format!("{}/v1/chat/completions", base_url)
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
                                if error_msg.contains("context_length_exceeded") {
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
impl LlmProvider for OpenAIProvider {
    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ProviderResponse> {
        let mut request_body = json!({
            "model": self.config.model,
            "messages": self.convert_messages(&request.messages),
            "stream": false,
        });
        
        // Add optional parameters
        if let Some(max_tokens) = request.max_tokens.or(self.config.max_tokens) {
            request_body["max_tokens"] = json!(max_tokens);
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
        
        let response: OpenAIResponse = self.execute_request(request_body).await?;
        
        let choice = response.choices.into_iter().next()
            .ok_or_else(|| LlmError::ApiError("No choices in response".to_string()))?;
        
        let content = choice.message.content.unwrap_or_default();
        let tool_calls = choice.message.tool_calls.unwrap_or_default()
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.function.name,
                arguments: tc.function.arguments,
            })
            .collect();
        
        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => Some(FinishReason::Stop),
            Some("length") => Some(FinishReason::Length),
            Some("content_filter") => Some(FinishReason::ContentFilter),
            Some("tool_calls") => Some(FinishReason::ToolCalls),
            _ => None,
        };
        
        Ok(ProviderResponse {
            content,
            tool_calls,
            usage: TokenUsage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
                total_tokens: response.usage.total_tokens,
            },
            finish_reason,
            metadata: HashMap::new(),
        })
    }
    
    async fn chat_completion_stream(
        &self,
        request: ChatRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = LlmResult<ProviderEvent>> + Send>>> {
        let mut request_body = json!({
            "model": self.config.model,
            "messages": self.convert_messages(&request.messages),
            "stream": true,
        });
        
        // Add optional parameters
        if let Some(max_tokens) = request.max_tokens.or(self.config.max_tokens) {
            request_body["max_tokens"] = json!(max_tokens);
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
                                if data == "[DONE]" {
                                    return Some(Ok(ProviderEvent::ContentStop));
                                }
                                
                                match serde_json::from_str::<OpenAIStreamResponse>(data) {
                                    Ok(stream_response) => {
                                        if let Some(choice) = stream_response.choices.first() {
                                            if let Some(delta) = &choice.delta {
                                                if let Some(content) = &delta.content {
                                                    return Some(Ok(ProviderEvent::ContentDelta {
                                                        delta: content.clone(),
                                                    }));
                                                }
                                                
                                                if let Some(tool_calls) = &delta.tool_calls {
                                                    for tool_call in tool_calls {
                                                        if let (Some(id), Some(function)) = (&tool_call.id, &tool_call.function) {
                                                            if let Some(name) = &function.name {
                                                                return Some(Ok(ProviderEvent::ToolUseStart {
                                                                    tool_call: ToolCall {
                                                                        id: id.clone(),
                                                                        name: name.clone(),
                                                                        arguments: function.arguments.clone().unwrap_or_default(),
                                                                    },
                                                                }));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
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
        "openai"
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

// OpenAI API types
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: OpenAIContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAIContent {
    String(String),
    Array(Vec<OpenAIContentBlock>),
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<OpenAIImageUrl>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIImageUrl {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: Option<OpenAIStreamDelta>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamToolCall {
    id: Option<String>,
    function: Option<OpenAIStreamFunction>,
}

#[derive(Debug, Deserialize)]
struct OpenAIStreamFunction {
    name: Option<String>,
    arguments: Option<serde_json::Value>,
}