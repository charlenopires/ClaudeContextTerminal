use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{debug, info};

use crate::{app::App, tui};
use crate::config::Config;
use super::run::RunCommand;

/// Crush - The glamourous AI coding agent for your favourite terminal 💘
#[derive(Parser)]
#[command(
    name = "crush",
    version,
    about = "The glamourous AI coding agent for your favourite terminal 💘",
    long_about = r#"Crush is an AI-powered terminal application that helps you with software development tasks.
It provides intelligent code assistance, documentation generation, and various development utilities.

Examples:
  crush                           # Start interactive mode
  crush run "explain this code"   # Run a single prompt
  crush --cwd /path/to/project    # Set working directory"#
)]
pub struct Cli {
    /// Current working directory
    #[arg(short = 'c', long = "cwd", global = true)]
    pub cwd: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short = 'd', long = "debug", global = true)]
    pub debug: bool,

    /// Dangerous mode - auto-accept all permissions
    #[arg(short = 'y', long = "yolo", global = true)]
    pub yolo: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a single prompt non-interactively
    Run(RunCommand),
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        // Set up debug logging if requested
        if self.debug {
            debug!("Debug logging enabled");
        }

        // Change working directory if specified
        if let Some(cwd) = &self.cwd {
            std::env::set_current_dir(cwd)
                .map_err(|e| anyhow::anyhow!("Failed to change directory to {}: {}", cwd.display(), e))?;
            info!("Changed working directory to: {}", cwd.display());
        }

        // Initialize configuration
        let config = Config::init().await?;
        debug!("Configuration initialized");

        match self.command {
            Some(Commands::Run(run_cmd)) => {
                // Execute non-interactive run command
                run_cmd.execute(&config, self.yolo).await
            }
            None => {
                // Start interactive mode
                self.start_interactive_mode(&config).await
            }
        }
    }

    async fn start_interactive_mode(&self, config: &Config) -> Result<()> {
        info!("Starting interactive mode");
        
        // Validate the configuration
        config.validate()?;
        
        // Setup signal handling for graceful shutdown
        self.setup_signal_handling().await;
        
        // Initialize the application
        let mut app = App::new(config.clone()).await?;
        
        // Start the application in interactive mode
        app.run_interactive().await?;
        
        info!("Application finished");
        Ok(())
    }

    async fn setup_signal_handling(&self) {
        // Set up signal handling for graceful shutdown
        tokio::spawn(async {
            #[cfg(unix)]
            {
                let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                    .expect("Failed to create SIGINT handler");
                let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("Failed to create SIGTERM handler");

                tokio::select! {
                    _ = sigint.recv() => {
                        info!("Received SIGINT, shutting down gracefully");
                    }
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, shutting down gracefully");
                    }
                }
            }
            
            #[cfg(windows)]
            {
                let mut ctrl_c = tokio::signal::windows::ctrl_c()
                    .expect("Failed to create Ctrl+C handler");
                let mut ctrl_break = tokio::signal::windows::ctrl_break()
                    .expect("Failed to create Ctrl+Break handler");

                tokio::select! {
                    _ = ctrl_c.recv() => {
                        info!("Received Ctrl+C, shutting down gracefully");
                    }
                    _ = ctrl_break.recv() => {
                        info!("Received Ctrl+Break, shutting down gracefully");
                    }
                }
            }
        });
    }
}