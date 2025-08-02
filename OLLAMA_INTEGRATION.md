# Ollama Integration Documentation

## Overview

This document describes the Ollama integration added to the Goofy application, enabling the use of local language models through the Ollama server.

## Features Added

### ✅ Complete Ollama Provider Implementation

- **Non-streaming chat completion**: Full support for single-response generation
- **Streaming chat completion**: Real-time response streaming with proper event handling
- **Model management**: List available models and health checks
- **Error handling**: Comprehensive error management with retries and proper error messages
- **Configuration**: Full integration with the existing config system

### ✅ Configuration Support

- **Environment variables**: `OLLAMA_HOST`, `OLLAMA_BASE_URL`, `GOOFY_PROVIDER`, `GOOFY_MODEL`
- **JSON configuration**: Added Ollama section to `goofy.example.json`
- **No API key required**: Automatically handles Ollama's keyless authentication

### ✅ CLI Integration

- **Provider selection**: Use `GOOFY_PROVIDER=ollama` to select Ollama
- **Model selection**: Use `GOOFY_MODEL=llama3.2` to specify the model
- **All existing CLI options**: Works with `--cwd`, `--debug`, `--quiet`, etc.

## Usage

### Prerequisites

1. Install Ollama: https://ollama.ai
2. Pull a model: `ollama pull llama3.2`
3. Start Ollama server: `ollama serve` (default: http://localhost:11434)

### Command Line Usage

```bash
# Using environment variables
GOOFY_PROVIDER=ollama GOOFY_MODEL=llama3.2 ./target/release/goofy run "Explain Rust ownership"

# Using configuration file
# Edit goofy.json to set default_provider: "ollama"
./target/release/goofy run "Generate a binary search function"

# Interactive mode with Ollama
GOOFY_PROVIDER=ollama ./target/release/goofy
```

### Configuration Examples

#### Environment Variables (.env)
```bash
GOOFY_PROVIDER=ollama
GOOFY_MODEL=llama3.2
OLLAMA_HOST=http://localhost:11434
```

#### JSON Configuration (goofy.json)
```json
{
  "llm": {
    "providers": {
      "ollama": {
        "base_url": "http://localhost:11434",
        "models": ["llama3.2", "mistral", "codellama"],
        "default_model": "llama3.2"
      }
    },
    "default_provider": "ollama"
  }
}
```

## Architecture

### Provider Implementation

The Ollama provider (`src/llm/ollama.rs`) implements the `LlmProvider` trait with:

- **API Endpoint Support**: `/api/chat` for conversations, `/api/tags` for model listing
- **Streaming**: Server-Sent Events parsing for real-time responses
- **Error Handling**: Proper HTTP error handling with meaningful messages
- **Type Safety**: Full integration with the existing type system

### Message Format Conversion

Converts between Goofy's internal message format and Ollama's API format:

```rust
// Goofy format -> Ollama format
Message::new_user("Hello") -> OllamaMessage { role: "user", content: "Hello" }
```

### Health Checking

Built-in health check functionality to verify Ollama server availability:

```rust
let is_healthy = provider.health_check().await?;
```

## Available Models

Common Ollama models that work with Goofy:

- **Code Models**: `codellama`, `deepseek-coder`, `starcoder2`
- **General Models**: `llama3.2`, `llama3.1`, `mistral`, `phi3`, `gemma2`
- **Specialized**: `sqlcoder`, `medllama2`

Pull models with: `ollama pull <model-name>`

## Error Handling

The integration provides clear error messages for common issues:

- **Server not running**: "HTTP error: Connection refused"
- **Model not found**: "model 'llama3.2' not found, try pulling it first"
- **Invalid configuration**: "Invalid configuration: model is required"

## Performance Considerations

- **Local Processing**: No network latency for API calls
- **Resource Usage**: Depends on model size and available system resources
- **Streaming**: Efficient real-time response delivery
- **Caching**: Ollama handles model loading and caching automatically

## Testing

Comprehensive test suite includes:

- **Unit tests**: Message conversion, response parsing, provider creation
- **Integration tests**: Full provider functionality with mock servers
- **Error scenarios**: Network failures, invalid responses, missing models

Run tests: `cargo test ollama`

## Troubleshooting

### Common Issues

1. **"Connection refused"**
   - Ensure Ollama is running: `ollama serve`
   - Check the correct port: default is 11434

2. **"Model not found"**
   - Pull the model: `ollama pull llama3.2`
   - Check available models: `ollama list`

3. **"Invalid configuration"**
   - Verify environment variables are set correctly
   - Check JSON configuration syntax

### Debugging

Enable debug logging to see detailed request/response information:

```bash
RUST_LOG=debug GOOFY_PROVIDER=ollama ./target/release/goofy run "test"
```

## Future Enhancements

Potential improvements for the Ollama integration:

- **Function Calling**: Add support for tool use when Ollama supports it
- **Model Auto-discovery**: Automatically detect available models
- **Performance Metrics**: Display token/second rates and model info
- **Custom Templates**: Support for model-specific prompt templates
- **Batch Processing**: Multi-prompt processing for efficiency

## Compatibility

- **Rust Version**: 1.70+
- **Ollama Version**: 0.1.0+
- **Supported Platforms**: All platforms supported by Ollama (Linux, macOS, Windows)
- **Models**: Any model supported by Ollama

The Ollama integration is now fully functional and provides a complete local LLM solution for the Goofy application.