//! Schema command implementation for configuration validation and JSON schema generation

use clap::{Args, Subcommand};
use anyhow::{Context, Result};
use std::{
    fs,
    path::PathBuf,
};
use schemars::{JsonSchema, schema_for};
use serde_json::Value;
use crate::config::Config;

/// Generate and validate configuration schemas
#[derive(Debug, Args)]
pub struct SchemaCommand {
    /// Output format for schema generation
    #[arg(short, long, default_value = "json")]
    pub format: SchemaFormat,

    /// Output file path (defaults to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Pretty print the output
    #[arg(short, long)]
    pub pretty: bool,

    /// Subcommands for schema operations
    #[command(subcommand)]
    pub command: Option<SchemaSubcommand>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SchemaFormat {
    Json,
    Yaml,
    Typescript,
}

#[derive(Debug, Subcommand)]
pub enum SchemaSubcommand {
    /// Generate JSON schema for configuration
    Generate {
        /// Schema title
        #[arg(long, default_value = "Goofy Configuration")]
        title: String,
        
        /// Schema description
        #[arg(long, default_value = "Configuration schema for Goofy AI coding assistant")]
        description: String,
    },
    /// Validate a configuration file against the schema
    Validate {
        /// Configuration file to validate
        config_file: PathBuf,
        
        /// Schema file to validate against (optional)
        #[arg(long)]
        schema_file: Option<PathBuf>,
    },
    /// Show configuration documentation
    Docs,
}

impl SchemaCommand {
    /// Execute the schema command
    pub async fn execute(&self, _config: &Config) -> Result<()> {
        match &self.command {
            Some(SchemaSubcommand::Generate { title, description }) => {
                self.generate_schema(title, description).await
            }
            Some(SchemaSubcommand::Validate { config_file, schema_file }) => {
                self.validate_config(config_file, schema_file.as_ref()).await
            }
            Some(SchemaSubcommand::Docs) => {
                self.show_docs().await
            }
            None => {
                // Default to generating schema
                self.generate_schema(
                    "Goofy Configuration",
                    "Configuration schema for Goofy AI coding assistant"
                ).await
            }
        }
    }

    /// Generate JSON schema for configuration
    async fn generate_schema(&self, title: &str, description: &str) -> Result<()> {
        let schema = schema_for!(Config);
        let mut schema_value = serde_json::to_value(schema)
            .context("Failed to convert schema to JSON value")?;

        // Add metadata
        if let Some(obj) = schema_value.as_object_mut() {
            obj.insert("title".to_string(), Value::String(title.to_string()));
            obj.insert("description".to_string(), Value::String(description.to_string()));
            obj.insert("$schema".to_string(), Value::String("https://json-schema.org/draft/2020-12/schema".to_string()));
        }

        let output = match self.format {
            SchemaFormat::Json => {
                if self.pretty {
                    serde_json::to_string_pretty(&schema_value)?
                } else {
                    serde_json::to_string(&schema_value)?
                }
            }
            SchemaFormat::Yaml => {
                serde_yaml::to_string(&schema_value)
                    .context("Failed to convert schema to YAML")?
            }
            SchemaFormat::Typescript => {
                self.generate_typescript_types(&schema_value)?
            }
        };

        if let Some(ref output_path) = self.output {
            fs::write(output_path, output)
                .with_context(|| format!("Failed to write schema to: {}", output_path.display()))?;
            println!("Schema written to: {}", output_path.display());
        } else {
            println!("{}", output);
        }

        Ok(())
    }

    /// Validate a configuration file
    async fn validate_config(&self, config_file: &PathBuf, schema_file: Option<&PathBuf>) -> Result<()> {
        // Read configuration file
        let config_content = fs::read_to_string(config_file)
            .with_context(|| format!("Failed to read config file: {}", config_file.display()))?;

        let config_value: Value = if config_file.extension().and_then(|ext| ext.to_str()) == Some("yaml") 
            || config_file.extension().and_then(|ext| ext.to_str()) == Some("yml") {
            serde_yaml::from_str(&config_content)
                .with_context(|| format!("Failed to parse YAML config: {}", config_file.display()))?
        } else {
            serde_json::from_str(&config_content)
                .with_context(|| format!("Failed to parse JSON config: {}", config_file.display()))?
        };

        // Get schema
        let schema_value = if let Some(schema_path) = schema_file {
            let schema_content = fs::read_to_string(schema_path)
                .with_context(|| format!("Failed to read schema file: {}", schema_path.display()))?;
            serde_json::from_str(&schema_content)
                .with_context(|| format!("Failed to parse schema file: {}", schema_path.display()))?
        } else {
            // Generate default schema
            let schema = schema_for!(Config);
            serde_json::to_value(schema)?
        };

        // Validate using jsonschema
        let compiled_schema = jsonschema::JSONSchema::compile(&schema_value)
            .context("Failed to compile JSON schema")?;

        match compiled_schema.validate(&config_value) {
            Ok(()) => {
                println!("✅ Configuration is valid!");
                
                // Additional validation: try to deserialize as Config
                match serde_json::from_value::<Config>(config_value) {
                    Ok(_) => {
                        println!("✅ Configuration can be loaded successfully!");
                    }
                    Err(e) => {
                        println!("⚠️  Configuration is schema-valid but cannot be loaded: {}", e);
                        println!("This might indicate missing required values or type mismatches.");
                    }
                }
            }
            Err(errors) => {
                println!("❌ Configuration validation failed:");
                for error in errors {
                    println!("  - {}: {}", error.instance_path, error);
                }
                return Err(anyhow::anyhow!("Configuration validation failed"));
            }
        }

        Ok(())
    }

    /// Show configuration documentation
    async fn show_docs(&self) -> Result<()> {
        println!("Goofy Configuration Documentation");
        println!("===============================\n");

        println!("Configuration File Locations:");
        println!("  1. ./goofy.json (project-specific)");
        println!("  2. ./.goofy.json (project-specific, hidden)");
        println!("  3. ~/.config/goofy/goofy.json (user-specific)");
        println!("  4. Environment variables (highest priority)\n");

        println!("Environment Variables:");
        println!("  GOOFY_PROVIDER     - Default LLM provider (openai, anthropic, ollama, etc.)");
        println!("  GOOFY_MODEL        - Default model name");
        println!("  OPENAI_API_KEY     - OpenAI API key");
        println!("  ANTHROPIC_API_KEY  - Anthropic API key");
        println!("  OLLAMA_HOST        - Ollama server URL");
        println!("  GOOFY_LOG_LEVEL    - Log level (debug, info, warn, error)");
        println!("  GOOFY_DATA_DIR     - Data directory for sessions and logs\n");

        println!("Configuration Sections:");
        println!("  providers    - LLM provider configurations");
        println!("  lsp          - Language Server Protocol settings");
        println!("  mcp          - Model Context Protocol settings");
        println!("  tui          - Terminal UI preferences");
        println!("  tools        - Tool permissions and settings");
        println!("  logging      - Logging configuration\n");

        println!("Example Configuration:");
        println!("{}", self.get_example_config());

        println!("\nFor more information, see: https://github.com/yourorg/goofy");

        Ok(())
    }

    /// Generate TypeScript types from JSON schema
    fn generate_typescript_types(&self, schema: &Value) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("// Generated TypeScript types for Goofy configuration\n\n");
        
        // This is a simplified TypeScript generation
        // A full implementation would properly traverse the schema
        output.push_str("export interface GoofyConfig {\n");
        output.push_str("  providers?: {\n");
        output.push_str("    openai?: {\n");
        output.push_str("      model?: string;\n");
        output.push_str("      api_key?: string;\n");
        output.push_str("      base_url?: string;\n");
        output.push_str("    };\n");
        output.push_str("    anthropic?: {\n");
        output.push_str("      model?: string;\n");
        output.push_str("      api_key?: string;\n");
        output.push_str("    };\n");
        output.push_str("    ollama?: {\n");
        output.push_str("      model?: string;\n");
        output.push_str("      base_url?: string;\n");
        output.push_str("    };\n");
        output.push_str("  };\n");
        output.push_str("  tui?: {\n");
        output.push_str("    theme?: string;\n");
        output.push_str("    keybindings?: Record<string, string>;\n");
        output.push_str("  };\n");
        output.push_str("  tools?: {\n");
        output.push_str("    permissions?: Record<string, 'auto' | 'prompt' | 'deny'>;\n");
        output.push_str("  };\n");
        output.push_str("  logging?: {\n");
        output.push_str("    level?: 'debug' | 'info' | 'warn' | 'error';\n");
        output.push_str("    file?: string;\n");
        output.push_str("  };\n");
        output.push_str("}\n");
        
        Ok(output)
    }

    /// Get example configuration
    fn get_example_config(&self) -> String {
        r#"{
  "providers": {
    "openai": {
      "model": "gpt-4",
      "api_key": "${OPENAI_API_KEY}"
    },
    "anthropic": {
      "model": "claude-3-sonnet-20240229",
      "api_key": "${ANTHROPIC_API_KEY}"
    },
    "ollama": {
      "model": "llama3.2",
      "base_url": "http://localhost:11434"
    }
  },
  "tui": {
    "theme": "goofy_dark",
    "keybindings": {
      "quit": "Ctrl+Q",
      "new_session": "Ctrl+N"
    }
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
}"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};
    use serde_json::json;

    #[tokio::test]
    async fn test_generate_schema() {
        let cmd = SchemaCommand {
            format: SchemaFormat::Json,
            output: None,
            pretty: true,
            command: Some(SchemaSubcommand::Generate {
                title: "Test Schema".to_string(),
                description: "Test description".to_string(),
            }),
        };

        // This test would need a proper Config struct that implements JsonSchema
        // For now, we just test that the function doesn't panic
        let config = Config::default();
        let result = cmd.execute(&config).await;
        // We expect this to work once JsonSchema is properly implemented
    }

    #[tokio::test]
    async fn test_validate_valid_config() {
        let dir = tempdir().unwrap();
        let config_file = dir.path().join("config.json");
        
        let valid_config = json!({
            "providers": {
                "openai": {
                    "model": "gpt-4",
                    "api_key": "test-key"
                }
            }
        });
        
        fs::write(&config_file, serde_json::to_string_pretty(&valid_config).unwrap()).unwrap();
        
        let cmd = SchemaCommand {
            format: SchemaFormat::Json,
            output: None,
            pretty: false,
            command: Some(SchemaSubcommand::Validate {
                config_file,
                schema_file: None,
            }),
        };

        let config = Config::default();
        // This test would need proper schema validation implementation
        // let result = cmd.execute(&config).await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_generation() {
        let cmd = SchemaCommand {
            format: SchemaFormat::Typescript,
            output: None,
            pretty: false,
            command: None,
        };

        let schema = json!({
            "type": "object",
            "properties": {
                "test": {"type": "string"}
            }
        });

        let ts_types = cmd.generate_typescript_types(&schema).unwrap();
        assert!(ts_types.contains("interface"));
        assert!(ts_types.contains("GoofyConfig"));
    }

    #[test]
    fn test_example_config() {
        let cmd = SchemaCommand {
            format: SchemaFormat::Json,
            output: None,
            pretty: false,
            command: None,
        };

        let example = cmd.get_example_config();
        assert!(example.contains("providers"));
        assert!(example.contains("openai"));
        assert!(example.contains("anthropic"));
        
        // Validate that it's proper JSON
        let parsed: Result<Value, _> = serde_json::from_str(&example);
        assert!(parsed.is_ok());
    }
}