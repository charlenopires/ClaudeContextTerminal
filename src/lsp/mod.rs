//! LSP (Language Server Protocol) integration for Goofy
//! 
//! This module provides integration with Language Server Protocol to enable
//! deep understanding of codebases through language servers.

pub mod client;
pub mod manager;
pub mod protocol;
pub mod types;

pub use client::LspClient;
pub use manager::LspManager;
pub use types::*;

use anyhow::Result;

/// Initialize LSP subsystem
pub async fn init() -> Result<LspManager> {
    let manager = LspManager::new().await?;
    Ok(manager)
}