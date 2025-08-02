//! LSP client implementation

use crate::lsp::{protocol::LspProtocol, types::*};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, BufReader, BufWriter},
    process::{Child, Command},
    sync::{mpsc, RwLock},
    time::timeout,
};
use tracing::{debug, error, info, trace, warn};

/// Response handler type for LSP requests
type ResponseHandler = tokio::sync::oneshot::Sender<Result<Value>>;

/// Notification handler type for LSP notifications
type NotificationHandler = Arc<dyn Fn(Value) -> Result<()> + Send + Sync>;

/// LSP client for communicating with a language server
pub struct LspClient {
    /// Language server process
    process: Option<Child>,
    
    /// Client configuration
    config: LspClientConfig,
    
    /// Language ID this client handles
    language_id: String,
    
    /// Next request ID
    next_id: AtomicI32,
    
    /// Pending response handlers
    response_handlers: Arc<RwLock<HashMap<i32, ResponseHandler>>>,
    
    /// Notification handlers
    notification_handlers: Arc<RwLock<HashMap<String, NotificationHandler>>>,
    
    /// Server capabilities after initialization
    capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    
    /// Open files tracking
    open_files: Arc<RwLock<HashMap<String, OpenFileInfo>>>,
    
    /// Diagnostics cache
    diagnostics: Arc<RwLock<HashMap<String, Vec<Diagnostic>>>>,
    
    /// Communication channels
    message_sender: Option<mpsc::UnboundedSender<LspMessage>>,
    shutdown_sender: Option<mpsc::UnboundedSender<()>>,
}

impl LspClient {
    /// Create a new LSP client
    pub fn new(language_id: String, config: LspClientConfig) -> Self {
        Self {
            process: None,
            config,
            language_id,
            next_id: AtomicI32::new(1),
            response_handlers: Arc::new(RwLock::new(HashMap::new())),
            notification_handlers: Arc::new(RwLock::new(HashMap::new())),
            capabilities: Arc::new(RwLock::new(None)),
            open_files: Arc::new(RwLock::new(HashMap::new())),
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            message_sender: None,
            shutdown_sender: None,
        }
    }

    /// Start the language server process
    pub async fn start(&mut self, workspace_root: Option<PathBuf>) -> Result<()> {
        info!("Starting LSP server for language: {}", self.language_id);
        
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args);
        
        if let Some(working_dir) = &self.config.working_dir {
            cmd.current_dir(working_dir);
        } else if let Some(root) = &workspace_root {
            cmd.current_dir(root);
        }
        
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut process = cmd.spawn()
            .map_err(|e| anyhow!("Failed to start LSP server '{}': {}", self.config.command, e))?;

        let stdin = process.stdin.take()
            .ok_or_else(|| anyhow!("Failed to get stdin for LSP process"))?;
        let stdout = process.stdout.take()
            .ok_or_else(|| anyhow!("Failed to get stdout for LSP process"))?;
        let stderr = process.stderr.take()
            .ok_or_else(|| anyhow!("Failed to get stderr for LSP process"))?;

        self.process = Some(process);

        // Start communication tasks
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel();
        
        self.message_sender = Some(msg_tx.clone());
        self.shutdown_sender = Some(shutdown_tx);

        // Start message handling tasks
        self.start_write_task(stdin, msg_rx, shutdown_rx).await?;
        self.start_read_task(stdout).await?;
        self.start_error_task(stderr).await?;

        // Initialize the server
        self.initialize(workspace_root).await?;
        
        info!("LSP server started successfully for language: {}", self.language_id);
        Ok(())
    }

    /// Stop the language server
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping LSP server for language: {}", self.language_id);
        
        // Send shutdown sequence
        if let Err(e) = self.shutdown().await {
            warn!("Error during LSP shutdown: {}", e);
        }

        // Signal shutdown to tasks
        if let Some(shutdown_tx) = &self.shutdown_sender {
            let _ = shutdown_tx.send(());
        }

        // Terminate process if still running
        if let Some(process) = &mut self.process {
            if let Err(e) = process.kill().await {
                warn!("Error killing LSP process: {}", e);
            }
        }

        self.process = None;
        self.message_sender = None;
        self.shutdown_sender = None;

        info!("LSP server stopped for language: {}", self.language_id);
        Ok(())
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.process.is_some() && self.message_sender.is_some()
    }

    /// Get server capabilities
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.read().await.clone()
    }

    /// Open a file in the language server
    pub async fn open_file(&self, uri: String, language_id: String, content: String) -> Result<()> {
        let version = 1;
        
        let file_info = OpenFileInfo {
            uri: uri.clone(),
            language_id: language_id.clone(),
            version,
            content: content.clone(),
        };

        // Store file info
        self.open_files.write().await.insert(uri.clone(), file_info);

        // Send did open notification
        let message = LspProtocol::create_did_open_notification(&uri, &language_id, version, &content);
        self.send_message(message).await?;

        debug!("Opened file in LSP: {}", uri);
        Ok(())
    }

    /// Close a file in the language server
    pub async fn close_file(&self, uri: &str) -> Result<()> {
        // Remove from open files
        self.open_files.write().await.remove(uri);

        // Send did close notification
        let message = LspProtocol::create_did_close_notification(uri);
        self.send_message(message).await?;

        debug!("Closed file in LSP: {}", uri);
        Ok(())
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(&self, uri: &str) -> Vec<Diagnostic> {
        self.diagnostics.read().await
            .get(uri)
            .cloned()
            .unwrap_or_default()
    }

    /// Send a request and wait for response
    async fn send_request(&self, method: String, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // Register response handler
        self.response_handlers.write().await.insert(id, tx);

        // Send request
        let message = LspMessage::Request { id, method, params };
        self.send_message(message).await?;

        // Wait for response with timeout
        match timeout(Duration::from_millis(5000), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(anyhow!("Response handler was dropped")),
            Err(_) => {
                // Remove handler on timeout
                self.response_handlers.write().await.remove(&id);
                Err(anyhow!("Request timed out"))
            }
        }
    }

    /// Send a message to the language server
    async fn send_message(&self, message: LspMessage) -> Result<()> {
        if let Some(sender) = &self.message_sender {
            sender.send(message)
                .map_err(|_| anyhow!("Failed to send message: channel closed"))?;
            Ok(())
        } else {
            Err(anyhow!("LSP client is not running"))
        }
    }

    /// Initialize the language server
    async fn initialize(&self, workspace_root: Option<PathBuf>) -> Result<()> {
        let root_uri = workspace_root
            .map(|p| format!("file://{}", p.display()));

        let capabilities = json!({
            "textDocument": {
                "synchronization": {
                    "didOpen": true,
                    "didClose": true,
                    "didChange": true
                },
                "hover": {
                    "contentFormat": ["markdown", "plaintext"]
                },
                "completion": {
                    "completionItem": {
                        "snippetSupport": true
                    }
                }
            },
            "workspace": {
                "workspaceFolders": true,
                "configuration": true
            }
        });

        let message = LspProtocol::create_initialize_request(
            self.next_id.fetch_add(1, Ordering::SeqCst),
            root_uri,
            capabilities,
        );

        self.send_message(message).await?;

        // Send initialized notification
        let initialized = LspProtocol::create_initialized_notification();
        self.send_message(initialized).await?;

        debug!("LSP server initialized for language: {}", self.language_id);
        Ok(())
    }

    /// Shutdown the language server gracefully
    async fn shutdown(&self) -> Result<()> {
        let shutdown_message = LspProtocol::create_shutdown_request(
            self.next_id.fetch_add(1, Ordering::SeqCst)
        );
        self.send_message(shutdown_message).await?;

        let exit_message = LspProtocol::create_exit_notification();
        self.send_message(exit_message).await?;

        Ok(())
    }

    /// Start the write task for sending messages
    async fn start_write_task<W: AsyncWrite + Unpin + Send + 'static>(
        &self,
        writer: W,
        mut msg_rx: mpsc::UnboundedReceiver<LspMessage>,
        mut shutdown_rx: mpsc::UnboundedReceiver<()>,
    ) -> Result<()> {
        tokio::spawn(async move {
            let mut writer = BufWriter::new(writer);
            
            loop {
                tokio::select! {
                    Some(message) = msg_rx.recv() => {
                        if let Err(e) = LspProtocol::write_message(&mut writer, &message).await {
                            error!("Failed to write LSP message: {}", e);
                            break;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        debug!("LSP write task shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Start the read task for receiving messages
    async fn start_read_task<R: AsyncRead + Unpin + Send + 'static>(&self, reader: R) -> Result<()> {
        let response_handlers = Arc::clone(&self.response_handlers);
        let notification_handlers = Arc::clone(&self.notification_handlers);
        let diagnostics = Arc::clone(&self.diagnostics);
        
        tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            
            loop {
                match LspProtocol::read_message(&mut reader).await {
                    Ok(message) => {
                        Self::handle_message(
                            message,
                            &response_handlers,
                            &notification_handlers,
                            &diagnostics,
                        ).await;
                    }
                    Err(e) => {
                        error!("Failed to read LSP message: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Start the error reading task
    async fn start_error_task<R: AsyncRead + Unpin + Send + 'static>(&self, stderr: R) -> Result<()> {
        let language_id = self.language_id.clone();
        
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        warn!("LSP server [{}] stderr: {}", language_id, line.trim());
                    }
                    Err(e) => {
                        error!("Error reading LSP stderr: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Handle incoming LSP messages
    async fn handle_message(
        message: LspMessage,
        response_handlers: &Arc<RwLock<HashMap<i32, ResponseHandler>>>,
        notification_handlers: &Arc<RwLock<HashMap<String, NotificationHandler>>>,
        diagnostics: &Arc<RwLock<HashMap<String, Vec<Diagnostic>>>>,
    ) {
        match message {
            LspMessage::Response { id, result, error } => {
                if let Some(handler) = response_handlers.write().await.remove(&id) {
                    let response = if let Some(error) = error {
                        Err(anyhow!("LSP error: {}", error.message))
                    } else {
                        Ok(result.unwrap_or(Value::Null))
                    };
                    
                    let _ = handler.send(response);
                }
            }
            LspMessage::Notification { method, params } => {
                // Handle built-in notifications
                if method == methods::TEXT_DOCUMENT_DIAGNOSTICS {
                    if let Some(ref params) = params {
                        Self::handle_diagnostics(params.clone(), diagnostics).await;
                    }
                }
                
                // Handle custom notification handlers
                if let Some(handler) = notification_handlers.read().await.get(&method) {
                    if let Some(params) = params {
                        if let Err(e) = handler(params) {
                            warn!("Notification handler error for {}: {}", method, e);
                        }
                    }
                }
            }
            LspMessage::Request { .. } => {
                // Server requests to client - not commonly used
                warn!("Received unexpected request from LSP server");
            }
        }
    }

    /// Handle diagnostic notifications
    async fn handle_diagnostics(
        params: Value,
        diagnostics: &Arc<RwLock<HashMap<String, Vec<Diagnostic>>>>,
    ) {
        // Parse diagnostics from LSP format
        if let Some(uri) = params.get("uri").and_then(|u| u.as_str()) {
            let mut parsed_diagnostics = Vec::new();
            
            if let Some(diag_array) = params.get("diagnostics").and_then(|d| d.as_array()) {
                for diag in diag_array {
                    if let Ok(diagnostic) = Self::parse_diagnostic(diag) {
                        parsed_diagnostics.push(diagnostic);
                    }
                }
            }
            
            diagnostics.write().await.insert(uri.to_string(), parsed_diagnostics);
            debug!("Updated diagnostics for: {}", uri);
        }
    }

    /// Parse a single diagnostic from LSP format
    fn parse_diagnostic(diag: &Value) -> Result<Diagnostic> {
        let range = diag.get("range")
            .ok_or_else(|| anyhow!("Missing range in diagnostic"))?;
        
        let start = range.get("start")
            .ok_or_else(|| anyhow!("Missing start in range"))?;
        
        let line = start.get("line")
            .and_then(|l| l.as_u64())
            .ok_or_else(|| anyhow!("Missing line in start"))? as u32;
        
        let character = start.get("character")
            .and_then(|c| c.as_u64())
            .ok_or_else(|| anyhow!("Missing character in start"))? as u32;

        let message = diag.get("message")
            .and_then(|m| m.as_str())
            .ok_or_else(|| anyhow!("Missing message in diagnostic"))?
            .to_string();

        let severity = diag.get("severity")
            .and_then(|s| s.as_u64())
            .map(|s| match s {
                1 => DiagnosticSeverity::Error,
                2 => DiagnosticSeverity::Warning,
                3 => DiagnosticSeverity::Information,
                _ => DiagnosticSeverity::Hint,
            });

        Ok(Diagnostic {
            message,
            severity,
            line,
            character,
            end_line: None, // Could be extracted from range.end if needed
            end_character: None,
            source: diag.get("source").and_then(|s| s.as_str()).map(|s| s.to_string()),
            code: diag.get("code").and_then(|c| c.as_str()).map(|c| c.to_string()),
        })
    }
}