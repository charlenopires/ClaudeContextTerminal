# Phase 4: CLI & Utilities

**Duration:** 1-2 weeks  
**Priority:** LOW  
**Target Completion:** 100% of total porting

## Overview

Phase 4 completes the porting process by implementing the remaining CLI commands, advanced utilities, and final polish features that make Goofy feature-complete with the original Crush.

## Critical Components

### 1. Advanced CLI Commands

**Location:** `src/cli/`

**Missing Commands:**
- `logs` - Log viewing and management
- `schema` - Configuration schema operations
- `config` - Configuration management
- `session` - Session management commands

```rust
// src/cli/logs.rs
pub struct LogsCommand {
    pub follow: bool,
    pub tail: Option<usize>,
    pub filter: Option<String>,
    pub format: LogFormat,
}

// src/cli/schema.rs
pub struct SchemaCommand {
    pub output_format: OutputFormat,
    pub validate_config: Option<PathBuf>,
}
```

### 2. Text Processing Utilities

**Location:** `src/utils/`

**Missing Utilities:**
- `diff.rs` - Text diffing and patch generation
- `format.rs` - Text formatting and processing
- `highlight.rs` - Syntax highlighting utilities
- `shell.rs` - Shell integration helpers

```rust
// src/utils/diff.rs
pub struct DiffEngine {
    pub algorithm: DiffAlgorithm,
    pub context_lines: usize,
}

pub enum DiffAlgorithm {
    Myers,
    Patience,
    Histogram,
}

// src/utils/highlight.rs
pub struct SyntaxHighlighter {
    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    pub current_theme: String,
}
```

### 3. Shell Integration

**Location:** `src/shell/`

**Shell Features:**
- Command history management
- Tab completion
- Shell-specific optimizations
- Environment integration

```rust
pub struct ShellIntegration {
    pub shell_type: ShellType,
    pub history_file: PathBuf,
    pub completion_scripts: Vec<CompletionScript>,
}

pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}
```

### 4. Advanced Configuration

**Location:** `src/config/`

**Configuration Enhancements:**
- Schema validation
- Configuration migration
- Environment detection
- Performance profiling

```rust
// src/config/schema.rs
pub struct ConfigSchema {
    pub version: String,
    pub definitions: HashMap<String, SchemaDefinition>,
    pub validation_rules: Vec<ValidationRule>,
}

// src/config/migration.rs
pub struct ConfigMigration {
    pub from_version: String,
    pub to_version: String,
    pub migration_steps: Vec<MigrationStep>,
}
```

## Implementation Steps

### Step 1: Advanced CLI Commands (Week 1)

1. **Logs command implementation**
   ```rust
   // src/cli/logs.rs
   impl LogsCommand {
       pub async fn execute(&self) -> Result<()> {
           let log_file = get_log_file_path()?;
           
           if self.follow {
               self.tail_logs(log_file).await
           } else {
               self.show_logs(log_file).await
           }
       }
   }
   ```

2. **Schema command**
   - JSON schema generation
   - Configuration validation
   - Documentation generation

3. **Session management CLI**
   - List sessions
   - Delete sessions
   - Export/import sessions
   - Session statistics

### Step 2: Text Processing Utilities (Week 1)

1. **Diff engine implementation**
   ```rust
   // src/utils/diff.rs
   pub fn generate_diff(old: &str, new: &str, algorithm: DiffAlgorithm) -> DiffResult {
       match algorithm {
           DiffAlgorithm::Myers => myers_diff(old, new),
           DiffAlgorithm::Patience => patience_diff(old, new),
           DiffAlgorithm::Histogram => histogram_diff(old, new),
       }
   }
   ```

2. **Enhanced formatting**
   - Code formatting
   - Markdown processing
   - Table formatting
   - JSON/YAML formatting

3. **Syntax highlighting**
   - Language detection
   - Theme management
   - Custom highlighting rules

### Step 3: Shell Integration (Week 1-2)

1. **Shell detection and setup**
   ```rust
   // src/shell/integration.rs
   pub fn detect_shell() -> ShellType {
       if let Ok(shell) = std::env::var("SHELL") {
           match shell.split('/').last() {
               Some("bash") => ShellType::Bash,
               Some("zsh") => ShellType::Zsh,
               Some("fish") => ShellType::Fish,
               _ => ShellType::Bash, // Default
           }
       } else {
           ShellType::Bash
       }
   }
   ```

2. **Completion script generation**
   - Bash completion
   - Zsh completion
   - Fish completion
   - PowerShell completion

3. **History integration**
   - Command history storage
   - Cross-session history
   - History search

### Step 4: Final Utilities (Week 2)

1. **Performance monitoring**
   ```rust
   // src/utils/profiler.rs
   pub struct PerformanceProfiler {
       pub start_time: Instant,
       pub checkpoints: Vec<ProfileCheckpoint>,
       pub memory_usage: MemoryTracker,
   }
   ```

2. **Environment helpers**
   - Environment variable management
   - Path utilities
   - Cross-platform compatibility

3. **Error reporting**
   - Enhanced error messages
   - Error recovery suggestions
   - Debug information collection

## Advanced CLI Features

### 1. Interactive Session Management

```bash
# List all sessions
goofy session list --format table

# Delete old sessions
goofy session clean --older-than 30d

# Export session
goofy session export session-id --format json

# Import session
goofy session import session.json

# Session statistics
goofy session stats --provider openai
```

### 2. Configuration Management

```bash
# Validate configuration
goofy config validate

# Show current configuration
goofy config show --format yaml

# Set configuration value
goofy config set provider.openai.model gpt-4

# Reset configuration
goofy config reset --section tui

# Generate shell completion
goofy completion bash > /etc/bash_completion.d/goofy
```

### 3. Log Management

```bash
# Follow logs in real-time
goofy logs --follow

# Show last 100 log entries
goofy logs --tail 100

# Filter logs by level
goofy logs --level error

# Export logs
goofy logs --export logs.json --since 2024-01-01
```

## Configuration Completion

### Final goofy.json Schema

```json
{
  "$schema": "https://goofy.dev/schema.json",
  "version": "1.0.0",
  "providers": {
    "openai": { "model": "gpt-4", "api_key": "${OPENAI_API_KEY}" },
    "anthropic": { "model": "claude-3-sonnet", "api_key": "${ANTHROPIC_API_KEY}" },
    "ollama": { "model": "llama3.2", "base_url": "http://localhost:11434" },
    "azure": { "endpoint": "${AZURE_ENDPOINT}", "api_key": "${AZURE_API_KEY}" },
    "bedrock": { "region": "us-east-1", "model": "anthropic.claude-3" },
    "gemini": { "api_key": "${GEMINI_API_KEY}", "model": "gemini-pro" },
    "vertex": { "project": "${GCP_PROJECT}", "location": "us-central1" }
  },
  "lsp": {
    "servers": {
      "rust": { "command": "rust-analyzer" },
      "typescript": { "command": "typescript-language-server", "args": ["--stdio"] },
      "python": { "command": "pylsp" }
    }
  },
  "mcp": {
    "servers": {
      "filesystem": {
        "transport": { "type": "stdio", "command": "mcp-fs-server" }
      }
    }
  },
  "tui": {
    "theme": "dark",
    "keybindings": { "quit": "Ctrl+Q" }
  },
  "tools": {
    "permissions": {
      "bash": "prompt",
      "edit": "auto",
      "file": "auto"
    }
  },
  "logging": {
    "level": "info",
    "file": "~/.goofy/logs/goofy.log"
  }
}
```

## Testing Strategy

### CLI Testing
- Command parsing validation
- Output format consistency
- Error handling completeness
- Cross-platform compatibility

### Utility Testing
- Diff algorithm correctness
- Syntax highlighting accuracy
- Shell integration reliability
- Performance benchmarks

### Integration Testing
- End-to-end workflows
- Configuration loading
- Error recovery
- Memory leak detection

## Success Criteria

- [ ] All CLI commands implemented
- [ ] Text processing utilities complete
- [ ] Shell integration working
- [ ] Configuration system finalized
- [ ] Documentation complete
- [ ] Performance optimized
- [ ] Cross-platform tested
- [ ] 100% feature parity with Crush

## Dependencies

**Final Crates:**
```toml
[dependencies]
# Diff algorithms
similar = "2.3"
diff = "0.1"

# Shell integration
shellexpand = "3.1"
clap_complete = "4.4"

# Text processing
textwrap = "0.16"
unicode-width = "0.1"

# Performance monitoring
pprof = "0.13"
jemallocator = "0.5"

# Cross-platform
dunce = "1.0"  # Path normalization
```

## Risk Mitigation

**Compatibility Risks:**
- Different shell environments
- Cross-platform path handling
- Terminal capability variations

**Performance Risks:**
- Large log files
- Memory usage in utilities
- Startup time optimization

**Maintenance Risks:**
- Configuration migration paths
- Backward compatibility
- Documentation maintenance

## Final Integration

**Complete Feature Matrix:**
- ✅ All LLM providers (7 total)
- ✅ Complete tool system (15+ tools)
- ✅ LSP integration
- ✅ MCP support
- ✅ Advanced TUI
- ✅ Full CLI suite
- ✅ Comprehensive configuration
- ✅ Shell integration
- ✅ Text processing utilities

**Quality Assurance:**
- Code coverage >90%
- Performance benchmarks
- Cross-platform testing
- Documentation complete
- Migration guides
- Example configurations

This phase delivers a production-ready Rust implementation that matches and potentially exceeds the original Crush's capabilities.