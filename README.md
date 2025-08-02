# Crush-RS

A Rust port of [Charmbracelet's Crush](https://github.com/charmbracelet/crush) - a terminal-based AI coding assistant.

## Features

- **Multi-LLM Support**: Works with OpenAI, Anthropic, and local Ollama models
- **Interactive TUI**: Modern terminal interface built with Ratatui
- **Session Management**: Persistent conversation history with SQLite
- **File System Integration**: Intelligent workspace traversal and file handling
- **Streaming Responses**: Real-time AI responses with proper error handling
- **Configuration Management**: JSON config files and environment variables

## Installation

### Prerequisites

- Rust 1.70+ 
- An API key from one of the supported providers:
  - OpenAI (GPT-4, GPT-3.5, etc.)
  - Anthropic (Claude-3 family)
  - Ollama (local models - no API key required)

### Building

```bash
git clone <this-repository>
cd ClaudeContextTerminal
cargo build --release
```

## Configuration

### Environment Variables

Copy `.env.example` to `.env` and add your API keys:

```bash
cp .env.example .env
# Edit .env with your API keys
```

For Ollama (local models), no API key is required, but you need to:

1. Install Ollama: https://ollama.ai
2. Pull a model: `ollama pull llama3.2`
3. Start Ollama server: `ollama serve` (usually runs on http://localhost:11434)

### Configuration File

Copy `crush.example.json` to `crush.json` and customize:

```bash
cp crush.example.json crush.json
# Edit crush.json with your preferences
```

## Usage

### Interactive Mode

Start the TUI interface:

```bash
./target/release/crush-rs
```

### Non-Interactive Mode

Run single prompts:

```bash
# Direct prompt
./target/release/crush-rs run "Explain Rust ownership"

# From stdin
echo "Generate a binary search function in Rust" | ./target/release/crush-rs run

# Quiet mode (no spinner)
./target/release/crush-rs run --quiet "Review this code"

# Using Ollama (local models)
CRUSH_PROVIDER=ollama CRUSH_MODEL=llama3.2 ./target/release/crush-rs run "Explain closures in Rust"
```

### Options

- `--cwd <path>`: Set working directory
- `--debug`: Enable debug logging
- `--yolo`: Auto-accept all permissions (dangerous!)

## Architecture

### Core Components

- **CLI**: Command-line interface using `clap`
- **TUI**: Terminal interface using `ratatui` and `crossterm`
- **LLM**: Provider abstraction for AI services
- **Session**: Conversation and history management
- **Config**: Configuration loading and validation
- **Utils**: File system and text processing utilities

### Dependencies

- **ratatui**: Terminal UI framework
- **clap**: Command-line argument parsing
- **tokio**: Async runtime
- **reqwest**: HTTP client for API calls
- **rusqlite**: SQLite database
- **serde**: JSON serialization
- **tracing**: Structured logging

## Development

### Running Tests

```bash
cargo test
```

### Debug Logging

```bash
RUST_LOG=debug ./target/release/crush-rs run "test prompt"
```

### Profiling

```bash
CRUSH_PROFILE=true ./target/release/crush-rs
# Profile server runs on http://localhost:6060
```

## Comparison to Original

This Rust port maintains the same functionality as the original Go version while leveraging Rust's:

- **Memory Safety**: No garbage collection, zero-cost abstractions
- **Performance**: Compiled binary with minimal runtime overhead
- **Concurrency**: Async/await with tokio for efficient I/O
- **Type Safety**: Strong typing prevents many runtime errors

### Go → Rust Mapping

| Go Library | Rust Equivalent | Purpose |
|------------|-----------------|---------|
| `cobra` | `clap` | CLI framework |
| `bubbletea` | `ratatui` | Terminal UI |
| `slog` | `tracing` | Structured logging |
| `godotenv` | `dotenvy` | Environment loading |
| `sqlite3` | `rusqlite` | Database |
| `http` | `reqwest` | HTTP client |

## License

MIT License - see original Charmbracelet Crush repository for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo fmt`
6. Submit a pull request

## Troubleshooting

### API Key Issues

Ensure your API keys are properly set:

```bash
# Check environment variables
echo $OPENAI_API_KEY
echo $ANTHROPIC_API_KEY

# Test with debug logging
RUST_LOG=debug ./target/release/crush-rs run "test"
```

### Ollama Issues

If using Ollama locally:

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Pull a model if needed
ollama pull llama3.2

# Test Ollama integration
CRUSH_PROVIDER=ollama CRUSH_MODEL=llama3.2 ./target/release/crush-rs run "Hello"
```

### Build Issues

Update dependencies:

```bash
cargo update
cargo build --release
```

### Database Issues

Reset session database:

```bash
rm ~/.crush/sessions.db
./target/release/crush-rs run "test"  # Recreates database
```