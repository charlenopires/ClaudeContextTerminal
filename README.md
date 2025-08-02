# Goofy

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
cd Goofy
cargo build --release
```

### Installing on macOS

After building the project, you can install the executable globally:

```bash
# Build the release binary
cargo build --release

# Copy to a directory in your PATH (choose one option)
# Option 1: Install to /usr/local/bin (requires sudo)
sudo cp target/release/goofy /usr/local/bin/goofy

# Option 2: Install to ~/.local/bin (user directory)
mkdir -p ~/.local/bin
cp target/release/goofy ~/.local/bin/goofy
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# Option 3: Install using Homebrew (if you have a tap)
# brew install your-tap/goofy

# Verify installation
which goofy
goofy --help
```

### Installing on Linux

After building the project, you can install the executable globally:

```bash
# Build the release binary
cargo build --release

# Copy to a directory in your PATH (choose one option)
# Option 1: Install to /usr/local/bin (requires sudo)
sudo cp target/release/goofy /usr/local/bin/goofy

# Option 2: Install to ~/.local/bin (user directory)
mkdir -p ~/.local/bin
cp target/release/goofy ~/.local/bin/goofy
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Option 3: Install using a package manager (if available)
# For Arch Linux (if you create an AUR package)
# yay -S goofy-git

# For Ubuntu/Debian (if you create a .deb package)
# sudo dpkg -i goofy_*.deb

# Verify installation
which goofy
goofy --help
```

### Installing on Windows

After building the project, you can install the executable:

```powershell
# Build the release binary
cargo build --release

# Option 1: Copy to a directory in your PATH
# Create a directory for the executable (if it doesn't exist)
mkdir C:\Users\%USERNAME%\bin

# Copy the executable
copy target\release\goofy.exe C:\Users\%USERNAME%\bin\goofy.exe

# Add to PATH (run as Administrator or add via System Properties)
setx PATH "%PATH%;C:\Users\%USERNAME%\bin"

# Option 2: Install to a system directory (requires Administrator)
copy target\release\goofy.exe C:\Windows\System32\goofy.exe

# Verify installation (restart PowerShell/CMD after adding to PATH)
where goofy
goofy --help
```

Alternatively, using Command Prompt:

```cmd
REM Build the release binary
cargo build --release

REM Copy to user directory
mkdir "%USERPROFILE%\bin"
copy target\release\goofy.exe "%USERPROFILE%\bin\goofy.exe"

REM Add to PATH
setx PATH "%PATH%;%USERPROFILE%\bin"

REM Verify installation
where goofy
goofy --help
```

Once installed on any platform, you can use `goofy` instead of the full path:

```bash
# Interactive mode
goofy

# Non-interactive mode
goofy run "Explain Rust ownership"

# With environment variables
GOOFY_PROVIDER=ollama GOOFY_MODEL=llama3.2 goofy run "test"
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

Copy `goofy.example.json` to `goofy.json` and customize:

```bash
cp goofy.example.json goofy.json
# Edit goofy.json with your preferences
```

## Usage

### Interactive Mode

Start the TUI interface:

```bash
./target/release/goofy
```

### Non-Interactive Mode

Run single prompts:

```bash
# Direct prompt
./target/release/goofy run "Explain Rust ownership"

# From stdin
echo "Generate a binary search function in Rust" | ./target/release/goofy run

# Quiet mode (no spinner)
./target/release/goofy run --quiet "Review this code"

# Using Ollama (local models)
GOOFY_PROVIDER=ollama GOOFY_MODEL=llama3.2 ./target/release/goofy run "Explain closures in Rust"
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
RUST_LOG=debug ./target/release/goofy run "test prompt"
```

### Profiling

```bash
GOOFY_PROFILE=true ./target/release/goofy
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
RUST_LOG=debug ./target/release/goofy run "test"
```

### Ollama Issues

If using Ollama locally:

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Pull a model if needed
ollama pull llama3.2

# Test Ollama integration
GOOFY_PROVIDER=ollama GOOFY_MODEL=llama3.2 ./target/release/goofy run "Hello"
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
rm ~/.goofy/sessions.db
./target/release/goofy run "test"  # Recreates database
```
