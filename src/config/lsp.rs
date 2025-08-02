use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LspConfig {
    #[serde(default)]
    pub lsp: HashMap<String, LspClientConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LspClientConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}