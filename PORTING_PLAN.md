# Goofy Porting Plan

This document outlines the complete plan for porting Charmbracelet's Crush (Go) to Goofy (Rust).

## Current Status: ~30% Complete

**Completed Components:**
- ✅ CLI with clap (equivalent to root.go/run.go)
- ✅ Basic configuration system
- ✅ LLM Providers (OpenAI, Anthropic, Ollama)
- ✅ SQLite-based sessions
- ✅ Basic TUI with Ratatui
- ✅ Logging/profiling infrastructure
- ✅ Event system
- ✅ Panic recovery

## Porting Phases

### Phase 1: Critical Foundations (2-3 weeks)
**Priority: MAXIMUM**
- Tools system (bash, file, edit, grep, rg, safe)
- Permission management system
- Database migration to SQLC pattern
- **Target:** 50% completion

### Phase 2: Providers & Protocols (2-3 weeks)
**Priority: HIGH**
- Additional LLM providers (Azure, Bedrock, Gemini, Vertex)
- Language Server Protocol (LSP) integration
- Model Context Protocol (MCP) support
- **Target:** 70% completion

### Phase 3: Complete TUI (3-4 weeks)
**Priority: MEDIUM**
- Advanced TUI components (sidebar, dialogs, completions)
- Image support and animations
- Responsive layouts and theming
- **Target:** 90% completion

### Phase 4: CLI & Utilities (1-2 weeks)
**Priority: LOW**
- Advanced CLI commands (logs, schema)
- Text utilities (diff, format, highlight)
- Shell integration
- **Target:** 100% completion

## Execution Commands

Use the following commands to work on each phase:

```bash
# Execute a specific phase
./target/release/goofy port phase1
./target/release/goofy port phase2
./target/release/goofy port phase3
./target/release/goofy port phase4

# Check phase status
./target/release/goofy port status
```

## Phase Details

Each phase has a detailed plan file:
- `docs/porting/phase1.md` - Critical Foundations
- `docs/porting/phase2.md` - Providers & Protocols
- `docs/porting/phase3.md` - Complete TUI
- `docs/porting/phase4.md` - CLI & Utilities

## Missing Critical Components

**Tools System (Phase 1):**
- `bash.rs` - Command execution
- `file.rs` - File operations
- `edit.rs` - File editing
- `grep.rs` - Text search
- `rg.rs` - Ripgrep integration
- `safe.rs` - Security validation

**LSP Integration (Phase 2):**
- Language server client management
- Workspace monitoring
- Auto-restart functionality

**MCP Support (Phase 2):**
- stdio/HTTP/SSE transport types
- Server lifecycle management

**Complete TUI (Phase 3):**
- Sidebar navigation
- Modal dialogs
- Image display support
- Advanced key bindings

## Estimated Total Time: 8-12 weeks

This plan ensures systematic porting of all Crush features while maintaining code quality and testing standards.