# Phase 1: Critical Foundations

**Duration:** 2-3 weeks  
**Priority:** MAXIMUM  
**Target Completion:** 50% of total porting

## Overview

Phase 1 focuses on implementing the core infrastructure that makes Goofy functional. Without these components, the application cannot perform its primary purpose as an AI coding assistant.

## Critical Components

### 1. Tools System Architecture

**Location:** `src/llm/tools/`

```rust
// Core trait that all tools must implement
pub trait BaseTool: Send + Sync {
    async fn execute(&self, request: ToolRequest) -> ToolResult<ToolResponse>;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value;
    fn requires_permission(&self) -> bool;
}

// Tool response structure
pub struct ToolResponse {
    pub content: String,
    pub success: bool,
    pub metadata: Option<serde_json::Value>,
}
```

**Files to Create:**
- `src/llm/tools/mod.rs` - Core tool management
- `src/llm/tools/bash.rs` - Command execution
- `src/llm/tools/file.rs` - File operations
- `src/llm/tools/edit.rs` - File editing
- `src/llm/tools/multiedit.rs` - Multiple file edits
- `src/llm/tools/grep.rs` - Text search
- `src/llm/tools/rg.rs` - Ripgrep integration
- `src/llm/tools/glob.rs` - Pattern matching
- `src/llm/tools/ls.rs` - Directory listing
- `src/llm/tools/safe.rs` - Security validation

### 2. Permission Management System

**Location:** `src/permission/`

```rust
pub enum PermissionLevel {
    Read,        // Read-only operations
    Write,       // File modifications
    Execute,     // Command execution
    Network,     // Network access
    Dangerous,   // Potentially harmful operations
}

pub struct PermissionManager {
    pub yolo_mode: bool,
    pub allowed_tools: HashSet<String>,
    pub denied_paths: HashSet<PathBuf>,
}
```

**Files to Create:**
- `src/permission/mod.rs` - Permission management
- `src/permission/validator.rs` - Safety validation
- `src/permission/tool.rs` - Tool-specific permissions

### 3. Database Migration to SQLC Pattern

**Location:** `src/db/`

**Migration from rusqlite to SQLC-like pattern:**

```rust
// Generated models (similar to Go's SQLC)
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub parent_id: Option<String>,
}

pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub token_count: Option<i32>,
}
```

**Files to Create:**
- `src/db/models.rs` - Generated models
- `src/db/queries.rs` - Type-safe queries
- `src/db/migrations/` - Schema evolution
- `src/db/connect.rs` - Connection management

## Implementation Steps

### Step 1: Tools System Foundation (Week 1)

1. **Create tool trait and management**
   ```bash
   # Create the basic structure
   touch src/llm/tools/mod.rs
   touch src/llm/tools/bash.rs
   touch src/llm/tools/file.rs
   ```

2. **Implement core tools**
   - Start with `bash.rs` (most critical)
   - Add `file.rs` for read operations
   - Implement `edit.rs` for modifications

3. **Integration with LLM providers**
   - Modify providers to support tool calls
   - Add tool response parsing

### Step 2: Permission System (Week 1-2)

1. **Create permission framework**
   ```bash
   mkdir src/permission
   touch src/permission/mod.rs
   ```

2. **Implement safety checks**
   - Path validation
   - Command sanitization
   - Tool authorization

3. **Integration with CLI**
   - Add `--yolo` mode support
   - Interactive permission prompts

### Step 3: Database Enhancement (Week 2-3)

1. **Create migration system**
   ```bash
   mkdir src/db/migrations
   ```

2. **Implement type-safe queries**
   - Replace rusqlite direct usage
   - Add prepared statements
   - Implement connection pooling

3. **Add advanced features**
   - Session relationships
   - Token tracking
   - Cost calculation

## Testing Strategy

### Unit Tests
- Tool execution with mock inputs
- Permission validation edge cases
- Database query correctness

### Integration Tests
- End-to-end tool workflows
- Permission system with real files
- Database migrations

### Safety Tests
- Malicious input handling
- Path traversal prevention
- Command injection protection

## Success Criteria

- [ ] All 10 core tools implemented and tested
- [ ] Permission system prevents dangerous operations
- [ ] Database supports all Crush schema features
- [ ] Tool calls work with all LLM providers
- [ ] Comprehensive test coverage (>80%)
- [ ] CLI integration complete

## Dependencies

**New Crates Needed:**
```toml
[dependencies]
# For command execution
tokio-process = "0.2"
which = "4.4"

# For file operations
tempfile = "3.8"
filetime = "0.2"

# For glob patterns
glob = "0.3"

# For process management
sysinfo = "0.29"
```

## Risk Mitigation

**Security Risks:**
- All file operations must validate paths
- Command execution requires sanitization
- Permission checks before any destructive operation

**Performance Risks:**
- Implement timeouts for all tool operations
- Add resource limits for command execution
- Use async operations to prevent blocking

**Compatibility Risks:**
- Ensure Windows/macOS/Linux compatibility
- Test with different shell environments
- Handle edge cases in file systems

## Next Phase Dependencies

Phase 2 depends on:
- Working tool system for LSP integration
- Permission framework for MCP security
- Enhanced database for session management

This phase is the foundation for all subsequent phases and must be completed before proceeding.