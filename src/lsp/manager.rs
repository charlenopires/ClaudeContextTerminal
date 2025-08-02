//! LSP manager for handling multiple language servers

use crate::{
    config::Config,
    lsp::{client::LspClient, types::*},
};
use anyhow::{anyhow, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manager for multiple LSP clients
pub struct LspManager {
    /// Active LSP clients by language ID
    clients: Arc<RwLock<HashMap<String, LspClient>>>,
    
    /// LSP configuration
    config: LspConfig,
    
    /// Current workspace root
    workspace_root: Option<PathBuf>,
}

impl LspManager {
    /// Create a new LSP manager
    pub async fn new() -> Result<Self> {
        Ok(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config: LspConfig::default(),
            workspace_root: None,
        })
    }

    /// Create LSP manager with configuration
    pub async fn with_config(config: LspConfig) -> Result<Self> {
        Ok(Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config,
            workspace_root: None,
        })
    }

    /// Set the workspace root directory
    pub async fn set_workspace_root<P: AsRef<Path>>(&mut self, root: P) -> Result<()> {
        let root_path = root.as_ref().to_path_buf();
        self.workspace_root = Some(root_path.clone());
        
        info!("Set LSP workspace root to: {}", root_path.display());
        
        // Restart any existing clients with the new workspace
        self.restart_all_clients().await?;
        
        Ok(())
    }

    /// Update LSP configuration
    pub async fn update_config(&mut self, config: LspConfig) -> Result<()> {
        self.config = config;
        
        // Restart clients to apply new configuration
        self.restart_all_clients().await?;
        
        Ok(())
    }

    /// Start a language server for the given language
    pub async fn start_language_server(&self, language_id: &str) -> Result<()> {
        if !self.config.settings.enabled {
            debug!("LSP is disabled globally");
            return Ok(());
        }

        let server_config = self.config.servers.get(language_id)
            .ok_or_else(|| anyhow!("No LSP server configured for language: {}", language_id))?;

        let mut client = LspClient::new(language_id.to_string(), server_config.clone());
        
        // Start the client
        client.start(self.workspace_root.clone()).await?;
        
        // Store the client
        self.clients.write().await.insert(language_id.to_string(), client);
        
        info!("Started LSP server for language: {}", language_id);
        Ok(())
    }

    /// Stop a language server
    pub async fn stop_language_server(&self, language_id: &str) -> Result<()> {
        if let Some(mut client) = self.clients.write().await.remove(language_id) {
            client.stop().await?;
            info!("Stopped LSP server for language: {}", language_id);
        }
        Ok(())
    }

    /// Get or start a language server for a file
    pub async fn get_or_start_server_for_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<String>> {
        let file_path = file_path.as_ref();
        
        // Determine language ID from file extension
        let language_id = self.detect_language(file_path)?;
        
        if let Some(lang_id) = &language_id {
            // Check if server is already running
            if !self.clients.read().await.contains_key(lang_id) {
                // Try to start the server
                if let Err(e) = self.start_language_server(lang_id).await {
                    warn!("Failed to start LSP server for {}: {}", lang_id, e);
                    return Ok(None);
                }
            }
        }
        
        Ok(language_id)
    }

    /// Open a file in the appropriate language server
    pub async fn open_file<P: AsRef<Path>>(&self, file_path: P, content: String) -> Result<()> {
        let file_path = file_path.as_ref();
        
        if let Some(language_id) = self.get_or_start_server_for_file(file_path).await? {
            let uri = Self::path_to_uri(file_path);
            
            if let Some(client) = self.clients.read().await.get(&language_id) {
                client.open_file(uri, language_id, content).await?;
                debug!("Opened file in LSP: {}", file_path.display());
            }
        }
        
        Ok(())
    }

    /// Close a file in the appropriate language server
    pub async fn close_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file_path = file_path.as_ref();
        let uri = Self::path_to_uri(file_path);
        
        // Find which language server has this file open
        let clients = self.clients.read().await;
        for client in clients.values() {
            // Try to close in all clients (no harm if not open)
            let _ = client.close_file(&uri).await;
        }
        
        debug!("Closed file in LSP: {}", file_path.display());
        Ok(())
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics<P: AsRef<Path>>(&self, file_path: P) -> Vec<Diagnostic> {
        let uri = Self::path_to_uri(file_path.as_ref());
        
        // Collect diagnostics from all language servers
        let mut all_diagnostics = Vec::new();
        let clients = self.clients.read().await;
        
        for client in clients.values() {
            let diagnostics = client.get_diagnostics(&uri).await;
            all_diagnostics.extend(diagnostics);
        }
        
        all_diagnostics
    }

    /// Get all active language servers
    pub async fn get_active_servers(&self) -> Vec<String> {
        self.clients.read().await.keys().cloned().collect()
    }

    /// Check if LSP is available for a language
    pub fn has_language_server(&self, language_id: &str) -> bool {
        self.config.servers.contains_key(language_id)
    }

    /// Get LSP configuration
    pub fn config(&self) -> &LspConfig {
        &self.config
    }

    /// Shutdown all language servers
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("Shutting down all LSP servers");
        
        let mut clients = self.clients.write().await;
        let client_names: Vec<String> = clients.keys().cloned().collect();
        
        for language_id in client_names {
            if let Some(mut client) = clients.remove(&language_id) {
                if let Err(e) = client.stop().await {
                    error!("Error stopping LSP server for {}: {}", language_id, e);
                }
            }
        }
        
        info!("All LSP servers shut down");
        Ok(())
    }

    /// Restart all active language servers
    async fn restart_all_clients(&self) -> Result<()> {
        let active_languages: Vec<String> = self.clients.read().await.keys().cloned().collect();
        
        // Stop all clients
        for language_id in &active_languages {
            if let Err(e) = self.stop_language_server(language_id).await {
                warn!("Error stopping LSP server for {}: {}", language_id, e);
            }
        }
        
        // Start them again
        for language_id in &active_languages {
            if let Err(e) = self.start_language_server(language_id).await {
                warn!("Error restarting LSP server for {}: {}", language_id, e);
            }
        }
        
        Ok(())
    }

    /// Detect language ID from file path
    fn detect_language<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<String>> {
        let file_path = file_path.as_ref();
        
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        // Check configured servers for matching file extensions
        for (language_id, server_config) in &self.config.servers {
            if server_config.file_extensions.contains(&extension.to_string()) {
                return Ok(Some(language_id.clone()));
            }
        }

        // Fallback to common mappings
        let language_id = match extension {
            "rs" => Some("rust"),
            "py" => Some("python"),
            "js" | "jsx" => Some("javascript"),
            "ts" | "tsx" => Some("typescript"),
            "go" => Some("go"),
            "java" => Some("java"),
            "cpp" | "cc" | "cxx" => Some("cpp"),
            "c" => Some("c"),
            "h" | "hpp" => Some("c"), // Could be cpp too
            "cs" => Some("csharp"),
            "php" => Some("php"),
            "rb" => Some("ruby"),
            "sh" | "bash" => Some("bash"),
            "json" => Some("json"),
            "yaml" | "yml" => Some("yaml"),
            "toml" => Some("toml"),
            "md" => Some("markdown"),
            "html" => Some("html"),
            "css" => Some("css"),
            "xml" => Some("xml"),
            _ => None,
        };

        Ok(language_id.map(|s| s.to_string()))
    }

    /// Convert file path to LSP URI
    fn path_to_uri<P: AsRef<Path>>(path: P) -> String {
        let path = path.as_ref();
        
        // Convert to absolute path
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .unwrap_or_default()
                .join(path)
        };

        format!("file://{}", absolute_path.display())
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            config: LspConfig::default(),
            workspace_root: None,
        }
    }
}

/// Load LSP configuration from app config
pub async fn load_lsp_config(config: &Config) -> LspConfig {
    // Use the existing LSP config from the main config
    config.lsp.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_language() {
        let manager = LspManager::new().await.unwrap();
        
        assert_eq!(
            manager.detect_language("test.rs").unwrap(),
            Some("rust".to_string())
        );
        
        assert_eq!(
            manager.detect_language("test.py").unwrap(),
            Some("python".to_string())
        );
        
        assert_eq!(
            manager.detect_language("test.unknown").unwrap(),
            None
        );
    }

    #[test]
    fn test_path_to_uri() {
        let uri = LspManager::path_to_uri("test.rs");
        assert!(uri.starts_with("file://"));
        assert!(uri.ends_with("test.rs"));
    }
}