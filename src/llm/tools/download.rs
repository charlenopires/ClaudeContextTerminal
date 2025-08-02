//! Download tool implementation for downloading files from URLs

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::{
    path::Path,
    time::Duration,
};
use tokio::{
    fs,
    io::AsyncWriteExt,
    time::timeout,
};
use reqwest::Client;

/// Download tool for downloading files from URLs
pub struct DownloadTool {
    client: Client,
}

impl DownloadTool {
    /// Create a new download tool
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // Default 5 minute timeout
            .user_agent("goofy/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl BaseTool for DownloadTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let url = request.parameters.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let file_path = request.parameters.get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: file_path"))?;

        let timeout_secs = request.parameters.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(300)
            .min(600); // Max 10 minutes

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some("URL must start with http:// or https://".to_string()),
            });
        }

        // Check permissions for network access
        if !request.permissions.allow_network && !request.permissions.yolo_mode {
            return Err(anyhow::anyhow!("Network access not permitted"));
        }

        // Check permissions for writing
        if !request.permissions.allow_write && !request.permissions.yolo_mode {
            return Err(anyhow::anyhow!("Write access not permitted"));
        }

        // Security check - validate path
        for restricted in &request.permissions.restricted_paths {
            if file_path.starts_with(restricted) && !request.permissions.yolo_mode {
                return Err(anyhow::anyhow!("Access to path '{}' is restricted", file_path));
            }
        }

        // Perform the download with timeout
        let download_timeout = Duration::from_secs(timeout_secs);
        match timeout(download_timeout, self.download_file(url, file_path)).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(e.to_string()),
            }),
            Err(_) => Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some("Download timed out".to_string()),
            }),
        }
    }

    fn name(&self) -> &str {
        "download"
    }

    fn description(&self) -> &str {
        r#"Downloads binary data from a URL and saves it to a local file.

WHEN TO USE THIS TOOL:
- Use when you need to download files, images, or other binary data from URLs
- Helpful for downloading assets, documents, or any file type
- Useful for saving remote content locally for processing or storage

HOW TO USE:
- Provide the URL to download from
- Specify the local file path where the content should be saved
- Optionally set a timeout for the request

FEATURES:
- Downloads any file type (binary or text)
- Automatically creates parent directories if they don't exist
- Handles large files efficiently with streaming
- Sets reasonable timeouts to prevent hanging
- Validates input parameters before making requests

LIMITATIONS:
- Maximum file size is 100MB
- Only supports HTTP and HTTPS protocols
- Cannot handle authentication or cookies
- Some websites may block automated requests
- Will overwrite existing files without warning

TIPS:
- Use absolute paths or paths relative to the working directory
- Set appropriate timeouts for large files or slow connections"#
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to download from"
                },
                "file_path": {
                    "type": "string",
                    "description": "The local file path where the downloaded content should be saved"
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in seconds (max 600)"
                }
            },
            "required": ["url", "file_path"]
        })
    }
}

impl DownloadTool {
    /// Download a file from URL to local path
    async fn download_file(&self, url: &str, file_path: &str) -> Result<ToolResponse, Box<dyn std::error::Error + Send + Sync>> {
        let path = Path::new(file_path);
        
        // Make the request
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Request failed with status code: {}", response.status())),
            });
        }

        // Check content length
        const MAX_SIZE: u64 = 100 * 1024 * 1024; // 100MB
        if let Some(content_length) = response.content_length() {
            if content_length > MAX_SIZE {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("File too large: {} bytes (max {} bytes)", content_length, MAX_SIZE)),
                });
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Create the output file
        let mut file = fs::File::create(path).await?;
        
        // Download and write the content with size limit
        let mut bytes_written = 0u64;
        let mut stream = response.bytes_stream();
        
        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            
            // Check size limit
            if bytes_written + chunk.len() as u64 > MAX_SIZE {
                // Clean up the incomplete file
                let _ = fs::remove_file(path).await;
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("File too large: exceeded {} bytes limit", MAX_SIZE)),
                });
            }
            
            file.write_all(&chunk).await?;
            bytes_written += chunk.len() as u64;
        }

        file.flush().await?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        let response_msg = if content_type != "unknown" {
            format!(
                "Successfully downloaded {} bytes to {} (Content-Type: {})",
                bytes_written,
                path.display(),
                content_type
            )
        } else {
            format!(
                "Successfully downloaded {} bytes to {}",
                bytes_written,
                path.display()
            )
        };

        let metadata = json!({
            "bytes_downloaded": bytes_written,
            "content_type": content_type,
            "file_path": file_path,
        });

        Ok(ToolResponse {
            content: response_msg,
            success: true,
            metadata: Some(metadata),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tools::ToolPermissions;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_download_tool_info() {
        let tool = DownloadTool::new();
        
        assert_eq!(tool.name(), "download");
        assert!(tool.description().contains("Downloads binary data"));
        
        let params = tool.parameters();
        assert!(params["properties"].get("url").is_some());
        assert!(params["properties"].get("file_path").is_some());
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let tool = DownloadTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("invalid-url"));
        params.insert("file_path".to_string(), json!("test.txt"));
        
        let request = ToolRequest {
            tool_name: "download".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: true,
                allow_write: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("URL must start with http://"));
    }

    #[tokio::test]
    async fn test_permission_denied_network() {
        let tool = DownloadTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com/file.txt"));
        params.insert("file_path".to_string(), json!("test.txt"));
        
        let request = ToolRequest {
            tool_name: "download".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: false,
                allow_write: true,
                yolo_mode: false,
                ..Default::default()
            },
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Network access not permitted"));
    }

    #[tokio::test]
    async fn test_permission_denied_write() {
        let tool = DownloadTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com/file.txt"));
        params.insert("file_path".to_string(), json!("test.txt"));
        
        let request = ToolRequest {
            tool_name: "download".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: true,
                allow_write: false,
                yolo_mode: false,
                ..Default::default()
            },
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Write access not permitted"));
    }
}