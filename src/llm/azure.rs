//! Azure OpenAI provider implementation (placeholder)

use crate::llm::{
    types::*,
    errors::{LlmError, LlmResult},
    provider::LlmProvider,
};
use std::collections::HashMap;
use futures::Stream;
use std::pin::Pin;
use tracing::info;

/// Azure OpenAI provider (simplified implementation)
pub struct AzureProvider {
    model: String,
    api_key: String,
    endpoint: String,
}

impl AzureProvider {
    /// Create provider from ProviderConfig
    pub fn from_config(config: ProviderConfig) -> LlmResult<Self> {
        let api_key = config.api_key
            .ok_or_else(|| LlmError::ConfigError("Azure API key is required".to_string()))?;

        let endpoint = config.base_url
            .ok_or_else(|| LlmError::ConfigError("Azure endpoint is required".to_string()))?;

        Ok(Self {
            model: config.model,
            api_key,
            endpoint,
        })
    }
}

#[async_trait::async_trait]
impl LlmProvider for AzureProvider {
    async fn chat_completion(&self, _request: ChatRequest) -> LlmResult<ProviderResponse> {
        // TODO: Implement actual Azure OpenAI API calls
        // This is a placeholder implementation
        Ok(ProviderResponse {
            content: "Azure provider placeholder response".to_string(),
            tool_calls: vec![],
            usage: TokenUsage {
                input_tokens: 0,
                output_tokens: 25,
                total_tokens: 25,
            },
            finish_reason: Some(FinishReason::Stop),
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn chat_completion_stream(
        &self,
        _request: ChatRequest,
    ) -> LlmResult<Pin<Box<dyn Stream<Item = LlmResult<ProviderEvent>> + Send>>> {
        // TODO: Implement streaming
        Err(LlmError::StreamError("Azure streaming not yet implemented".to_string()))
    }

    fn name(&self) -> &str {
        "azure"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn validate_config(&self) -> LlmResult<()> {
        if self.api_key.is_empty() {
            return Err(LlmError::ConfigError("Azure API key is required".to_string()));
        }

        if self.endpoint.is_empty() {
            return Err(LlmError::ConfigError("Azure endpoint is required".to_string()));
        }

        info!("Azure OpenAI provider configuration validated successfully");
        Ok(())
    }
}