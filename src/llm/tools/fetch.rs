//! Fetch tool implementation for downloading web content

use super::{BaseTool, ToolRequest, ToolResponse, ToolResult};
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;
use tokio::time::timeout;
use reqwest::Client;

/// Fetch tool for downloading content from URLs
pub struct FetchTool {
    client: Client,
}

impl FetchTool {
    /// Create a new fetch tool
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("goofy/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }
}

#[async_trait]
impl BaseTool for FetchTool {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse> {
        let url = request.parameters.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        let format = request.parameters.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("text")
            .to_lowercase();

        let timeout_secs = request.parameters.get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(30)
            .min(120); // Max 2 minutes

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some("URL must start with http:// or https://".to_string()),
            });
        }

        if !["text", "markdown", "html"].contains(&format.as_str()) {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some("Format must be one of: text, markdown, html".to_string()),
            });
        }

        // Check permissions for network access
        if !request.permissions.allow_network && !request.permissions.yolo_mode {
            return Err(anyhow::anyhow!("Network access not permitted"));
        }

        // Perform the fetch with timeout
        let fetch_timeout = Duration::from_secs(timeout_secs);
        match timeout(fetch_timeout, self.fetch_content(url, &format)).await {
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
                error: Some("Fetch timed out".to_string()),
            }),
        }
    }

    fn name(&self) -> &str {
        "fetch"
    }

    fn description(&self) -> &str {
        r#"Fetches content from a URL and returns it in the specified format.

WHEN TO USE THIS TOOL:
- Use when you need to download content from a URL
- Helpful for retrieving documentation, API responses, or web content
- Useful for getting external information to assist with tasks

HOW TO USE:
- Provide the URL to fetch content from
- Specify the desired output format (text, markdown, or html)
- Optionally set a timeout for the request

FEATURES:
- Supports three output formats: text, markdown, and html
- Automatically handles HTTP redirects
- Sets reasonable timeouts to prevent hanging
- Validates input parameters before making requests

LIMITATIONS:
- Maximum response size is 5MB
- Only supports HTTP and HTTPS protocols
- Cannot handle authentication or cookies
- Some websites may block automated requests

TIPS:
- Use text format for plain text content or simple API responses
- Use markdown format for content that should be rendered with formatting
- Use html format when you need the raw HTML structure
- Set appropriate timeouts for potentially slow websites"#
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                },
                "format": {
                    "type": "string",
                    "description": "The format to return the content in (text, markdown, or html)",
                    "enum": ["text", "markdown", "html"]
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in seconds (max 120)"
                }
            },
            "required": ["url", "format"]
        })
    }
}

impl FetchTool {
    /// Fetch content from URL and format it
    async fn fetch_content(&self, url: &str, format: &str) -> Result<ToolResponse, Box<dyn std::error::Error + Send + Sync>> {
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
        const MAX_SIZE: u64 = 5 * 1024 * 1024; // 5MB
        if let Some(content_length) = response.content_length() {
            if content_length > MAX_SIZE {
                return Ok(ToolResponse {
                    content: String::new(),
                    success: false,
                    metadata: None,
                    error: Some(format!("Response too large: {} bytes (max {} bytes)", content_length, MAX_SIZE)),
                });
            }
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        // Read the response body
        let bytes = response.bytes().await?;
        
        // Check size limit
        if bytes.len() as u64 > MAX_SIZE {
            return Ok(ToolResponse {
                content: String::new(),
                success: false,
                metadata: None,
                error: Some(format!("Response too large: {} bytes (max {} bytes)", bytes.len(), MAX_SIZE)),
            });
        }

        let content = String::from_utf8(bytes.to_vec()).map_err(|_| {
            "Response content is not valid UTF-8"
        })?;

        // Format the content based on the requested format
        let formatted_content = match format {
            "text" => {
                if content_type.contains("text/html") {
                    self.extract_text_from_html(&content)?
                } else {
                    content
                }
            }
            "markdown" => {
                if content_type.contains("text/html") {
                    let markdown = self.convert_html_to_markdown(&content)?;
                    format!("```\n{}\n```", markdown)
                } else {
                    format!("```\n{}\n```", content)
                }
            }
            "html" => {
                if content_type.contains("text/html") {
                    // Extract body content from HTML
                    self.extract_body_from_html(&content)?
                } else {
                    content
                }
            }
            _ => content,
        };

        // Truncate if too large for display
        const MAX_DISPLAY_SIZE: usize = 100_000; // 100KB for display
        let final_content = if formatted_content.len() > MAX_DISPLAY_SIZE {
            format!(
                "{}\n\n[Content truncated to {} bytes]",
                &formatted_content[..MAX_DISPLAY_SIZE],
                MAX_DISPLAY_SIZE
            )
        } else {
            formatted_content
        };

        let metadata = json!({
            "url": url,
            "format": format,
            "content_type": content_type,
            "content_length": bytes.len(),
            "truncated": final_content.len() != content.len(),
        });

        Ok(ToolResponse {
            content: final_content,
            success: true,
            metadata: Some(metadata),
            error: None,
        })
    }

    /// Extract text content from HTML
    fn extract_text_from_html(&self, html: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Simple HTML text extraction using html2md
        let text = html2md::parse_html(html);
        Ok(text.chars().filter(|c| !c.is_control() || c.is_whitespace()).collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "))
    }

    /// Convert HTML to Markdown
    fn convert_html_to_markdown(&self, html: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Use html2md for HTML to Markdown conversion
        let markdown = html2md::parse_html(html);
        Ok(markdown)
    }

    /// Extract body content from HTML
    fn extract_body_from_html(&self, html: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Simple body extraction using scraper
        use scraper::{Html, Selector};
        
        let document = Html::parse_document(html);
        let body_selector = Selector::parse("body").map_err(|_| "Failed to create body selector")?;
        
        if let Some(body) = document.select(&body_selector).next() {
            Ok(format!("<html>\n<body>\n{}\n</body>\n</html>", body.inner_html()))
        } else {
            // If no body tag found, return the full HTML
            Ok(html.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::tools::ToolPermissions;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_fetch_tool_info() {
        let tool = FetchTool::new();
        
        assert_eq!(tool.name(), "fetch");
        assert!(tool.description().contains("Fetches content from a URL"));
        
        let params = tool.parameters();
        assert!(params["properties"].get("url").is_some());
        assert!(params["properties"].get("format").is_some());
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let tool = FetchTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("invalid-url"));
        params.insert("format".to_string(), json!("text"));
        
        let request = ToolRequest {
            tool_name: "fetch".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("URL must start with http://"));
    }

    #[tokio::test]
    async fn test_invalid_format() {
        let tool = FetchTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        params.insert("format".to_string(), json!("invalid"));
        
        let request = ToolRequest {
            tool_name: "fetch".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: true,
                ..Default::default()
            },
        };
        
        let response = tool.execute(request).await.unwrap();
        assert!(!response.success);
        assert!(response.error.as_ref().unwrap().contains("Format must be one of"));
    }

    #[tokio::test]
    async fn test_permission_denied() {
        let tool = FetchTool::new();
        
        let mut params = HashMap::new();
        params.insert("url".to_string(), json!("https://example.com"));
        params.insert("format".to_string(), json!("text"));
        
        let request = ToolRequest {
            tool_name: "fetch".to_string(),
            parameters: params,
            working_directory: None,
            permissions: ToolPermissions {
                allow_network: false,
                yolo_mode: false,
                ..Default::default()
            },
        };
        
        let result = tool.execute(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Network access not permitted"));
    }
}