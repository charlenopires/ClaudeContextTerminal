//! Base completion provider trait and registry

use super::{CompletionItem, CompletionContext};
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

/// Base trait for all completion providers
#[async_trait]
pub trait CompletionProvider: Send + Sync + Debug {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get completion items for the given context
    async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>>;

    /// Check if this provider should be used for the given context
    fn is_applicable(&self, context: &CompletionContext) -> bool {
        let _ = context;
        true // Default: applicable to all contexts
    }

    /// Get provider priority for the given context (higher = more important)
    fn get_priority(&self, context: &CompletionContext) -> i32 {
        let _ = context;
        0 // Default neutral priority
    }

    /// Check if provider supports caching
    fn supports_caching(&self) -> bool {
        true // Most providers benefit from caching
    }

    /// Get cache TTL in seconds (None = use default)
    fn cache_ttl(&self) -> Option<u64> {
        None // Use default cache TTL
    }

    /// Called when provider is registered
    async fn on_register(&self) -> Result<()> {
        Ok(())
    }

    /// Called when provider is unregistered
    async fn on_unregister(&self) -> Result<()> {
        Ok(())
    }
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Maximum number of items to return
    pub max_items: usize,
    
    /// Enable fuzzy matching
    pub fuzzy_matching: bool,
    
    /// Minimum query length to trigger
    pub min_query_length: usize,
    
    /// Provider-specific settings
    pub settings: serde_json::Value,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            max_items: 50,
            fuzzy_matching: true,
            min_query_length: 1,
            settings: serde_json::Value::Null,
        }
    }
}

/// Enhanced provider trait with configuration support
#[async_trait]
pub trait ConfigurableProvider: CompletionProvider {
    /// Get current configuration
    fn get_config(&self) -> &ProviderConfig;
    
    /// Update configuration
    async fn update_config(&mut self, config: ProviderConfig) -> Result<()>;
    
    /// Validate configuration
    fn validate_config(&self, config: &ProviderConfig) -> Result<()> {
        let _ = config;
        Ok(())
    }
}

/// File system completion provider
#[derive(Debug)]
pub struct FileCompletionProvider {
    config: ProviderConfig,
    max_depth: usize,
    show_hidden: bool,
}

impl FileCompletionProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig::default(),
            max_depth: 5,
            show_hidden: false,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_hidden_files(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }
}

#[async_trait]
impl CompletionProvider for FileCompletionProvider {
    fn name(&self) -> &str {
        "file"
    }

    async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>> {
        // This will be implemented in file_provider.rs
        Ok(Vec::new())
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.is_file_path()
    }

    fn get_priority(&self, context: &CompletionContext) -> i32 {
        if context.is_file_path() { 10 } else { 0 }
    }
}

/// Command completion provider
#[derive(Debug)]
pub struct CommandCompletionProvider {
    config: ProviderConfig,
    available_commands: Vec<String>,
}

impl CommandCompletionProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig::default(),
            available_commands: Vec::new(),
        }
    }

    pub fn with_commands(mut self, commands: Vec<String>) -> Self {
        self.available_commands = commands;
        self
    }
}

#[async_trait]
impl CompletionProvider for CommandCompletionProvider {
    fn name(&self) -> &str {
        "command"
    }

    async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>> {
        // This will be implemented in command_provider.rs
        Ok(Vec::new())
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.is_command()
    }

    fn get_priority(&self, context: &CompletionContext) -> i32 {
        if context.is_command() { 8 } else { 0 }
    }
}

/// History-based completion provider
#[derive(Debug)]
pub struct HistoryCompletionProvider {
    config: ProviderConfig,
    max_history: usize,
}

impl HistoryCompletionProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig::default(),
            max_history: 100,
        }
    }

    pub fn with_max_history(mut self, max_history: usize) -> Self {
        self.max_history = max_history;
        self
    }
}

#[async_trait]
impl CompletionProvider for HistoryCompletionProvider {
    fn name(&self) -> &str {
        "history"
    }

    async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>> {
        // This will be implemented in history_provider.rs
        Ok(Vec::new())
    }

    fn get_priority(&self, _context: &CompletionContext) -> i32 {
        5 // Medium priority
    }
}

/// Code completion provider (LSP-based)
#[derive(Debug)]
pub struct CodeCompletionProvider {
    config: ProviderConfig,
    supported_languages: Vec<String>,
}

impl CodeCompletionProvider {
    pub fn new() -> Self {
        Self {
            config: ProviderConfig::default(),
            supported_languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
            ],
        }
    }

    pub fn with_languages(mut self, languages: Vec<String>) -> Self {
        self.supported_languages = languages;
        self
    }
}

#[async_trait]
impl CompletionProvider for CodeCompletionProvider {
    fn name(&self) -> &str {
        "code"
    }

    async fn get_completions(&self, context: &CompletionContext) -> Result<Vec<CompletionItem>> {
        // This will be implemented in code_provider.rs
        Ok(Vec::new())
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        if let Some(language) = &context.language {
            self.supported_languages.contains(language)
        } else {
            false
        }
    }

    fn get_priority(&self, context: &CompletionContext) -> i32 {
        if self.is_applicable(context) { 15 } else { 0 }
    }

    fn supports_caching(&self) -> bool {
        false // Code completions are highly contextual
    }
}

/// Provider registry for managing multiple providers
#[derive(Debug)]
pub struct ProviderRegistry {
    providers: Vec<Box<dyn CompletionProvider>>,
    enabled_providers: std::collections::HashSet<String>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            enabled_providers: std::collections::HashSet::new(),
        }
    }

    /// Register a new provider
    pub async fn register(&mut self, provider: Box<dyn CompletionProvider>) -> Result<()> {
        let name = provider.name().to_string();
        provider.on_register().await?;
        self.providers.push(provider);
        self.enabled_providers.insert(name);
        Ok(())
    }

    /// Unregister a provider by name
    pub async fn unregister(&mut self, name: &str) -> Result<()> {
        if let Some(pos) = self.providers.iter().position(|p| p.name() == name) {
            let provider = self.providers.remove(pos);
            provider.on_unregister().await?;
            self.enabled_providers.remove(name);
        }
        Ok(())
    }

    /// Enable or disable a provider
    pub fn set_enabled(&mut self, name: &str, enabled: bool) {
        if enabled {
            self.enabled_providers.insert(name.to_string());
        } else {
            self.enabled_providers.remove(name);
        }
    }

    /// Check if a provider is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.enabled_providers.contains(name)
    }

    /// Get all applicable providers for the given context
    pub fn get_applicable_providers(&self, context: &CompletionContext) -> Vec<&dyn CompletionProvider> {
        let mut applicable: Vec<_> = self.providers
            .iter()
            .filter(|p| self.is_enabled(p.name()) && p.is_applicable(context))
            .map(|p| p.as_ref())
            .collect();

        // Sort by priority (highest first)
        applicable.sort_by(|a, b| b.get_priority(context).cmp(&a.get_priority(context)));
        applicable
    }

    /// Get list of all provider names
    pub fn provider_names(&self) -> Vec<String> {
        self.providers.iter().map(|p| p.name().to_string()).collect()
    }

    /// Get list of enabled provider names
    pub fn enabled_provider_names(&self) -> Vec<String> {
        self.providers
            .iter()
            .filter(|p| self.is_enabled(p.name()))
            .map(|p| p.name().to_string())
            .collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestProvider {
        name: String,
        priority: i32,
    }

    #[async_trait]
    impl CompletionProvider for TestProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn get_completions(&self, _context: &CompletionContext) -> Result<Vec<CompletionItem>> {
            Ok(vec![CompletionItem::new("test", "test", &self.name)])
        }

        fn get_priority(&self, _context: &CompletionContext) -> i32 {
            self.priority
        }
    }

    #[tokio::test]
    async fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();
        
        let provider1 = Box::new(TestProvider {
            name: "test1".to_string(),
            priority: 10,
        });
        let provider2 = Box::new(TestProvider {
            name: "test2".to_string(),
            priority: 5,
        });

        registry.register(provider1).await.unwrap();
        registry.register(provider2).await.unwrap();

        assert_eq!(registry.provider_names().len(), 2);
        assert!(registry.is_enabled("test1"));
        assert!(registry.is_enabled("test2"));

        // Test priority ordering
        let context = CompletionContext::default();
        let applicable = registry.get_applicable_providers(&context);
        assert_eq!(applicable.len(), 2);
        assert_eq!(applicable[0].name(), "test1"); // Higher priority first
        assert_eq!(applicable[1].name(), "test2");
    }

    #[tokio::test]
    async fn test_provider_enable_disable() {
        let mut registry = ProviderRegistry::new();
        
        let provider = Box::new(TestProvider {
            name: "test".to_string(),
            priority: 10,
        });

        registry.register(provider).await.unwrap();
        assert!(registry.is_enabled("test"));

        registry.set_enabled("test", false);
        assert!(!registry.is_enabled("test"));

        let context = CompletionContext::default();
        let applicable = registry.get_applicable_providers(&context);
        assert_eq!(applicable.len(), 0); // Disabled provider should not be included
    }
}