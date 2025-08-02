use anyhow::{anyhow, Result};
use clap::Args;
use std::io::{self, Read};
use tracing::{debug, info};

use crate::app::App;
use crate::config::Config;

/// Run a single prompt non-interactively
#[derive(Args)]
pub struct RunCommand {
    /// The prompt to run. If not provided, will read from stdin
    pub prompt: Vec<String>,

    /// Suppress spinner and other interactive elements
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,
}

impl RunCommand {
    pub async fn execute(&self, config: &Config, yolo: bool) -> Result<()> {
        debug!("Executing run command");

        // Get the prompt either from arguments or stdin
        let prompt = self.get_prompt()?;
        
        if prompt.trim().is_empty() {
            return Err(anyhow!("No prompt provided. Use arguments or pipe input via stdin."));
        }

        info!("Running prompt: {}", prompt.chars().take(50).collect::<String>());

        // Validate the configuration
        config.validate()?;

        // Initialize the application in non-interactive mode
        let mut app = App::new(config.clone()).await?;
        
        // Run the prompt non-interactively
        let result = app.run_non_interactive(&prompt, self.quiet).await?;
        
        // Output the result
        println!("{}", result);
        
        Ok(())
    }

    fn get_prompt(&self) -> Result<String> {
        if !self.prompt.is_empty() {
            // Join all arguments into a single prompt
            Ok(self.prompt.join(" "))
        } else {
            // Read from stdin
            debug!("Reading prompt from stdin");
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)
                .map_err(|e| anyhow!("Failed to read from stdin: {}", e))?;
            Ok(buffer)
        }
    }
}