use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, collections::HashMap};
use tracing::debug;

/// Application configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// Current working directory
    pub cwd: PathBuf,
    
    /// Data directory for storing sessions and databases
    pub data_dir: PathBuf,
    
    /// AI provider type
    pub provider: String,
    
    /// API key
    pub api_key: Option<String>,
    
    /// Base URL for the API
    pub base_url: Option<String>,
    
    /// Model to use
    pub model: String,
    
    /// Maximum tokens for responses
    pub max_tokens: Option<u32>,
    
    /// Temperature for sampling
    pub temperature: Option<f32>,
    
    /// Top-p for nucleus sampling
    pub top_p: Option<f32>,
    
    /// Whether to stream responses
    pub stream: bool,
    
    /// Extra headers for API requests
    pub extra_headers: HashMap<String, String>,
    
    /// Extra body parameters for API requests
    pub extra_body: HashMap<String, serde_json::Value>,
    
    /// System message for conversations
    pub system_message: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            data_dir: PathBuf::from("./data"),
            provider: "openai".to_string(),
            api_key: None,
            base_url: None,
            model: "gpt-4".to_string(),
            max_tokens: Some(4096),
            temperature: Some(0.7),
            top_p: None,
            stream: true,
            extra_headers: HashMap::new(),
            extra_body: HashMap::new(),
            system_message: None,
        }
    }
}

impl Config {
    /// Initialize configuration from various sources
    pub async fn init() -> Result<Self> {
        debug!("Initializing configuration");
        
        let mut config = Self::default();
        
        // Load from environment variables
        config.load_from_env();
        
        // Try to load from configuration files
        if let Ok(file_config) = Self::load_from_file().await {
            config.merge_with(file_config);
        }
        
        // Ensure data directory exists
        if !config.data_dir.exists() {
            std::fs::create_dir_all(&config.data_dir)?;
        }
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    pub fn load_from_env(&mut self) {
        if let Ok(provider) = std::env::var("CRUSH_PROVIDER") {
            self.provider = provider;
        }
        
        // Check for provider-specific API keys
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if self.provider == "openai" && self.api_key.is_none() {
                self.api_key = Some(key);
            }
        }
        
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if self.provider == "anthropic" && self.api_key.is_none() {
                self.api_key = Some(key);
            }
        }
        
        // Ollama doesn't require API keys, but we check for base URL
        if self.provider == "ollama" {
            if let Ok(url) = std::env::var("OLLAMA_HOST") {
                self.base_url = Some(url);
            } else if let Ok(url) = std::env::var("OLLAMA_BASE_URL") {
                self.base_url = Some(url);
            }
            // Ollama doesn't need an API key
            if self.api_key.is_none() {
                self.api_key = Some("not-required".to_string());
            }
        }
        
        // Generic API key
        if let Ok(key) = std::env::var("CRUSH_API_KEY") {
            self.api_key = Some(key);
        }
        
        if let Ok(base_url) = std::env::var("CRUSH_BASE_URL") {
            self.base_url = Some(base_url);
        }
        
        if let Ok(model) = std::env::var("CRUSH_MODEL") {
            self.model = model;
        }
        
        if let Ok(max_tokens_str) = std::env::var("CRUSH_MAX_TOKENS") {
            if let Ok(max_tokens) = max_tokens_str.parse() {
                self.max_tokens = Some(max_tokens);
            }
        }
        
        if let Ok(temp_str) = std::env::var("CRUSH_TEMPERATURE") {
            if let Ok(temperature) = temp_str.parse() {
                self.temperature = Some(temperature);
            }
        }
        
        if let Ok(stream_str) = std::env::var("CRUSH_STREAM") {
            self.stream = stream_str.to_lowercase() == "true";
        }
        
        if let Ok(data_dir) = std::env::var("CRUSH_DATA_DIR") {
            self.data_dir = PathBuf::from(data_dir);
        }
        
        if let Ok(system_message) = std::env::var("CRUSH_SYSTEM_MESSAGE") {
            self.system_message = Some(system_message);
        }
    }
    
    /// Load configuration from crush.json files
    pub async fn load_from_file() -> Result<Self> {
        // Configuration priority (as per Crush documentation):
        // 1. ./.crush.json
        // 2. ./crush.json
        // 3. $HOME/.config/crush/crush.json
        
        let mut config_paths = vec![
            PathBuf::from("./.crush.json"),
            PathBuf::from("./crush.json"),
        ];
        
        if let Some(config_dir) = dirs::config_dir() {
            config_paths.push(config_dir.join("crush").join("crush.json"));
        }
        
        for path in config_paths {
            if path.exists() {
                debug!("Loading configuration from: {}", path.display());
                let content = tokio::fs::read_to_string(&path).await?;
                let config: Self = serde_json::from_str(&content)?;
                return Ok(config);
            }
        }
        
        // Check for existing crush.json in current directory
        let crush_json = PathBuf::from("./crush.json");
        if crush_json.exists() {
            debug!("Loading configuration from: {}", crush_json.display());
            let content = tokio::fs::read_to_string(&crush_json).await?;
            let config: Self = serde_json::from_str(&content)?;
            return Ok(config);
        }
        
        Err(anyhow::anyhow!("No configuration file found"))
    }
    
    /// Merge another configuration into this one
    pub fn merge_with(&mut self, other: Self) {
        if other.api_key.is_some() {
            self.api_key = other.api_key;
        }
        if other.base_url.is_some() {
            self.base_url = other.base_url;
        }
        if !other.model.is_empty() && other.model != "gpt-4" {
            self.model = other.model;
        }
        if other.max_tokens.is_some() {
            self.max_tokens = other.max_tokens;
        }
        if other.temperature.is_some() {
            self.temperature = other.temperature;
        }
        if other.top_p.is_some() {
            self.top_p = other.top_p;
        }
        if !other.extra_headers.is_empty() {
            self.extra_headers.extend(other.extra_headers);
        }
        if !other.extra_body.is_empty() {
            self.extra_body.extend(other.extra_body);
        }
        if other.system_message.is_some() {
            self.system_message = other.system_message;
        }
    }
    
    /// Check if the configuration has a valid API key
    pub fn has_api_key(&self) -> bool {
        // Ollama doesn't require API keys
        if self.provider == "ollama" {
            return true;
        }
        self.api_key.is_some() && !self.api_key.as_ref().unwrap().is_empty()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if !self.has_api_key() {
            return Err(anyhow::anyhow!(
                "No API key configured. Set OPENAI_API_KEY, ANTHROPIC_API_KEY, or CRUSH_API_KEY environment variable. For Ollama, no API key is required."
            ));
        }
        
        if self.model.is_empty() {
            return Err(anyhow::anyhow!("Model is required"));
        }
        
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 {
                return Err(anyhow::anyhow!("max_tokens must be greater than 0"));
            }
        }
        
        if let Some(temperature) = self.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(anyhow::anyhow!("temperature must be between 0.0 and 2.0"));
            }
        }
        
        if let Some(top_p) = self.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(anyhow::anyhow!("top_p must be between 0.0 and 1.0"));
            }
        }
        
        Ok(())
    }
}