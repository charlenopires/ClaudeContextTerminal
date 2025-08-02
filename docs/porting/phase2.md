# Phase 2: Providers & Protocols

**Duration:** 2-3 weeks  
**Priority:** HIGH  
**Target Completion:** 70% of total porting

## Overview

Phase 2 extends Goofy's capabilities by adding missing LLM providers and implementing the two major protocol systems that make Crush powerful: Language Server Protocol (LSP) and Model Context Protocol (MCP).

## Critical Components

### 1. Additional LLM Providers

**Location:** `src/llm/`

**Missing Providers:**
- Azure OpenAI (`azure.rs`)
- AWS Bedrock (`bedrock.rs`) 
- Google Gemini (`gemini.rs`)
- Google Vertex AI (`vertex.rs`)

```rust
// Provider trait consistency
impl LlmProvider for AzureProvider {
    async fn chat_completion(&self, request: ChatRequest) -> LlmResult<ProviderResponse>;
    async fn chat_completion_stream(&self, request: ChatRequest) -> LlmResult<Pin<Box<dyn Stream<Item = LlmResult<ProviderEvent>> + Send>>>;
    fn name(&self) -> &str { "azure" }
    fn model(&self) -> &str { &self.config.model }
    fn validate_config(&self) -> LlmResult<()>;
}
```

### 2. Language Server Protocol (LSP)

**Location:** `src/lsp/`

LSP enables Goofy to understand codebases at a deep level by communicating with language servers.

```rust
pub struct LspClient {
    pub language: String,
    pub server_path: String,
    pub workspace_path: PathBuf,
    pub client: Option<tokio::process::Child>,
    pub capabilities: Option<ServerCapabilities>,
}

pub struct LspManager {
    pub clients: HashMap<String, LspClient>,
    pub workspace_watcher: Option<RecommendedWatcher>,
}
```

**Key Features:**
- Auto-start language servers for different file types
- Workspace monitoring and auto-restart
- Diagnostics integration
- Symbol navigation
- Code completion

### 3. Model Context Protocol (MCP)

**Location:** `src/mcp/`

MCP allows Goofy to connect to external tools and services.

```rust
pub enum McpTransport {
    Stdio { command: String, args: Vec<String> },
    Http { url: String, headers: HashMap<String, String> },
    Sse { url: String, headers: HashMap<String, String> },
}

pub struct McpServer {
    pub name: String,
    pub transport: McpTransport,
    pub client: Option<McpClient>,
    pub tools: Vec<McpTool>,
}
```

**Transport Types:**
- **stdio:** Direct process communication
- **HTTP:** REST API communication  
- **SSE:** Server-sent events for real-time updates

## Implementation Steps

### Step 1: Azure OpenAI Provider (Week 1)

1. **Create Azure provider**
   ```rust
   // src/llm/azure.rs
   pub struct AzureProvider {
       pub config: AzureConfig,
       pub client: reqwest::Client,
   }
   
   pub struct AzureConfig {
       pub api_key: String,
       pub endpoint: String,
       pub api_version: String,
       pub deployment_name: String,
   }
   ```

2. **Azure-specific authentication**
   - API key authentication
   - Endpoint URL handling
   - API version management

### Step 2: AWS Bedrock Provider (Week 1)

1. **AWS SDK integration**
   ```toml
   [dependencies]
   aws-config = "0.55"
   aws-sdk-bedrock = "0.28"
   aws-credential-types = "0.55"
   ```

2. **Bedrock-specific features**
   - IAM authentication
   - Model invocation
   - Streaming support

### Step 3: Google Providers (Week 1-2)

1. **Gemini provider**
   - Google AI Studio API
   - API key authentication
   - Safety settings

2. **Vertex AI provider**
   - Google Cloud authentication
   - Project/region configuration
   - Service account support

### Step 4: LSP Implementation (Week 2-3)

1. **LSP client foundation**
   ```rust
   // src/lsp/client.rs
   pub struct LspClient {
       stdin: tokio::process::ChildStdin,
       stdout: tokio::process::ChildStdout,
       message_id: AtomicU64,
   }
   ```

2. **Language server management**
   - Auto-detection of language servers
   - Configuration from goofy.json
   - Process lifecycle management

3. **Workspace integration**
   - File watching with notify
   - Automatic restart on crashes
   - Multi-language support

### Step 5: MCP Implementation (Week 3)

1. **MCP transport layer**
   ```rust
   // src/mcp/transport.rs
   pub trait McpTransport {
       async fn send_request(&mut self, request: McpRequest) -> McpResult<McpResponse>;
       async fn start(&mut self) -> McpResult<()>;
       async fn stop(&mut self) -> McpResult<()>;
   }
   ```

2. **Server management**
   - Server lifecycle
   - Tool discovery
   - Error handling and recovery

## Configuration Extensions

### Enhanced goofy.json Schema

```json
{
  "providers": {
    "azure": {
      "api_key": "${AZURE_OPENAI_API_KEY}",
      "endpoint": "https://your-resource.openai.azure.com/",
      "api_version": "2023-12-01-preview",
      "deployment_name": "gpt-4"
    },
    "bedrock": {
      "region": "us-east-1",
      "model": "anthropic.claude-3-sonnet-20240229-v1:0"
    }
  },
  "lsp": {
    "rust": {
      "command": "rust-analyzer",
      "args": [],
      "workspace": true
    },
    "typescript": {
      "command": "typescript-language-server",
      "args": ["--stdio"]
    }
  },
  "mcp": {
    "servers": {
      "filesystem": {
        "transport": {
          "type": "stdio",
          "command": "mcp-filesystem-server",
          "args": ["--path", "/project"]
        }
      },
      "web-search": {
        "transport": {
          "type": "http",
          "url": "http://localhost:8080/mcp",
          "headers": {
            "Authorization": "Bearer ${WEB_SEARCH_TOKEN}"
          }
        }
      }
    }
  }
}
```

## Testing Strategy

### Provider Tests
- Authentication flow testing
- Streaming response handling
- Error condition simulation
- Rate limiting compliance

### LSP Tests
- Language server startup/shutdown
- Message protocol compliance
- Workspace change detection
- Multi-language scenarios

### MCP Tests
- Transport reliability
- Server failure recovery
- Tool execution validation
- Security boundary testing

## Success Criteria

- [ ] All 4 additional providers implemented
- [ ] LSP integration with 3+ language servers
- [ ] MCP support for all 3 transport types
- [ ] Configuration schema complete
- [ ] Provider switching works seamlessly
- [ ] LSP workspace monitoring functional
- [ ] MCP server lifecycle management
- [ ] Comprehensive test coverage

## Dependencies

**New Crates:**
```toml
[dependencies]
# AWS Bedrock
aws-config = "0.55"
aws-sdk-bedrock = "0.28"

# LSP support
lsp-types = "0.97"
lsp-server = "0.7"

# File watching
notify = "6.1"

# HTTP client enhancements
tower = "0.4"
tower-http = "0.4"

# Process management
tokio-process = "0.2"
```

## Risk Mitigation

**Authentication Complexity:**
- Each provider has different auth mechanisms
- Implement comprehensive error handling
- Provide clear setup documentation

**LSP Stability:**
- Language servers can crash or hang
- Implement timeout and restart logic
- Monitor resource usage

**MCP Security:**
- External tools can be dangerous
- Implement permission boundaries
- Validate all tool responses

## Integration Points

**With Phase 1:**
- Tools system must support LSP diagnostics
- Permission system must handle MCP tools
- Database must store LSP/MCP configurations

**With Phase 3:**
- TUI must display LSP diagnostics
- UI needs MCP tool selection
- Status indicators for server health

This phase significantly enhances Goofy's intelligence and connectivity, making it a true rival to the original Crush.