//! Error types for LLM providers

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    
    #[error("Authentication failed: {0}")]
    AuthError(String),
    
    #[error("Context limit exceeded: {0}")]
    ContextLimitError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
    
    #[error("Stream error: {0}")]
    StreamError(String),
    
    #[error("Tool call error: {0}")]
    ToolCallError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type LlmResult<T> = Result<T, LlmError>;