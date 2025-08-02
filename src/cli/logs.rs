//! Logs command implementation for viewing and following Goofy logs

use clap::{Args, Subcommand};
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::PathBuf,
    time::Duration,
};
use tokio::{
    fs,
    time::{interval, sleep},
};
use notify::{Watcher, RecursiveMode, recommended_watcher};
use serde_json::Value;
use crate::config::Config;

/// View and manage Goofy logs
#[derive(Debug, Args)]
pub struct LogsCommand {
    /// Follow log output in real-time
    #[arg(short, long)]
    pub follow: bool,

    /// Number of lines to show from the end of the log
    #[arg(short, long, default_value = "100")]
    pub tail: usize,

    /// Filter logs by level (debug, info, warn, error)
    #[arg(short, long)]
    pub level: Option<String>,

    /// Export logs to a file
    #[arg(short, long)]
    pub export: Option<PathBuf>,

    /// Show logs since a specific date (YYYY-MM-DD)
    #[arg(long)]
    pub since: Option<String>,

    /// Show logs until a specific date (YYYY-MM-DD)
    #[arg(long)]
    pub until: Option<String>,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub format: LogFormat,

    /// Subcommands for log management
    #[command(subcommand)]
    pub command: Option<LogsSubcommand>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum LogsSubcommand {
    /// Clear all logs
    Clear,
    /// Archive old logs
    Archive {
        /// Archive logs older than specified days
        #[arg(long, default_value = "30")]
        older_than_days: u32,
    },
    /// Show log statistics
    Stats,
}

impl LogsCommand {
    /// Execute the logs command
    pub async fn execute(&self, config: &Config) -> Result<()> {
        if let Some(ref command) = self.command {
            return self.handle_subcommand(command, config).await;
        }

        let log_file = self.get_log_file_path(config)?;
        
        if !log_file.exists() {
            eprintln!("No log file found at: {}", log_file.display());
            eprintln!("Make sure Goofy has been run at least once to generate logs.");
            return Ok(());
        }

        if self.follow {
            self.follow_logs(&log_file).await
        } else {
            self.show_logs(&log_file).await
        }
    }

    /// Handle subcommands
    async fn handle_subcommand(&self, command: &LogsSubcommand, config: &Config) -> Result<()> {
        match command {
            LogsSubcommand::Clear => {
                let log_file = self.get_log_file_path(config)?;
                if log_file.exists() {
                    fs::write(&log_file, "").await
                        .with_context(|| format!("Failed to clear log file: {}", log_file.display()))?;
                    println!("Log file cleared: {}", log_file.display());
                } else {
                    println!("No log file found to clear.");
                }
                Ok(())
            }
            LogsSubcommand::Archive { older_than_days } => {
                self.archive_logs(config, *older_than_days).await
            }
            LogsSubcommand::Stats => {
                self.show_log_stats(config).await
            }
        }
    }

    /// Get the log file path from configuration
    fn get_log_file_path(&self, config: &Config) -> Result<PathBuf> {
        let log_dir = config.data_dir.join("logs");
        Ok(log_dir.join("goofy.log"))
    }

    /// Show logs from file
    async fn show_logs(&self, log_file: &PathBuf) -> Result<()> {
        let file = File::open(log_file)
            .with_context(|| format!("Failed to open log file: {}", log_file.display()))?;
        
        let lines = self.read_tail_lines(file)?;
        let filtered_lines = self.filter_lines(lines)?;

        if let Some(ref export_path) = self.export {
            self.export_logs(&filtered_lines, export_path).await?;
            println!("Logs exported to: {}", export_path.display());
            return Ok(());
        }

        self.print_lines(&filtered_lines);

        if filtered_lines.len() == self.tail {
            eprintln!("\nShowing last {} lines. Use --tail to show more or --follow to watch new entries.", self.tail);
        }

        Ok(())
    }

    /// Follow logs in real-time
    async fn follow_logs(&self, log_file: &PathBuf) -> Result<()> {
        // First show existing tail lines
        if log_file.exists() {
            let file = File::open(log_file)
                .with_context(|| format!("Failed to open log file: {}", log_file.display()))?;
            
            let lines = self.read_tail_lines(file)?;
            let filtered_lines = self.filter_lines(lines)?;
            
            if !filtered_lines.is_empty() {
                self.print_lines(&filtered_lines);
                println!("\n--- Following new log entries ---\n");
            }
        }

        // Set up file watcher
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        
        let log_file_clone = log_file.clone();
        let _watcher = recommended_watcher(move |res| {
            if let Ok(event) = res {
                if event.paths.iter().any(|p| p == &log_file_clone) {
                    let _ = tx.try_send(());
                }
            }
        })?;

        // Watch the log file directory
        let log_dir = log_file.parent().unwrap_or_else(|| std::path::Path::new("."));
        _watcher.watch(log_dir, RecursiveMode::NonRecursive)?;

        let mut last_position = if log_file.exists() {
            fs::metadata(log_file).await?.len()
        } else {
            0
        };

        // Poll for new content
        let mut interval = interval(Duration::from_millis(500));
        
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    // File changed, check for new content
                    if let Ok(new_lines) = self.read_new_lines(log_file, &mut last_position).await {
                        if !new_lines.is_empty() {
                            let filtered_lines = self.filter_lines(new_lines)?;
                            self.print_lines(&filtered_lines);
                        }
                    }
                }
                _ = interval.tick() => {
                    // Periodic check for new content (fallback)
                    if let Ok(new_lines) = self.read_new_lines(log_file, &mut last_position).await {
                        if !new_lines.is_empty() {
                            let filtered_lines = self.filter_lines(new_lines)?;
                            self.print_lines(&filtered_lines);
                        }
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("\nStopping log follow...");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Read the last N lines from a file
    fn read_tail_lines(&self, mut file: File) -> Result<Vec<String>> {
        let reader = BufReader::new(&mut file);
        
        let all_lines: Vec<String> = reader.lines()
            .collect::<std::io::Result<Vec<_>>>()
            .context("Failed to read lines from log file")?;

        let lines = if all_lines.len() <= self.tail {
            all_lines
        } else {
            let skip_count = all_lines.len() - self.tail;
            all_lines.into_iter()
                .skip(skip_count)
                .collect()
        };

        Ok(lines)
    }

    /// Read new lines from a specific position
    async fn read_new_lines(&self, log_file: &PathBuf, last_position: &mut u64) -> Result<Vec<String>> {
        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let current_size = fs::metadata(log_file).await?.len();
        
        if current_size <= *last_position {
            return Ok(Vec::new());
        }

        let mut file = File::open(log_file)?;
        file.seek(SeekFrom::Start(*last_position))?;
        
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .collect::<std::io::Result<Vec<_>>>()?;

        *last_position = current_size;
        
        Ok(lines)
    }

    /// Filter lines based on criteria
    fn filter_lines(&self, lines: Vec<String>) -> Result<Vec<String>> {
        let mut filtered = lines;

        // Filter by log level
        if let Some(ref level) = self.level {
            filtered = filtered.into_iter()
                .filter(|line| self.line_matches_level(line, level))
                .collect();
        }

        // Filter by date range
        if self.since.is_some() || self.until.is_some() {
            filtered = filtered.into_iter()
                .filter(|line| self.line_matches_date_range(line))
                .collect();
        }

        Ok(filtered)
    }

    /// Check if a log line matches the specified level
    fn line_matches_level(&self, line: &str, level: &str) -> bool {
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<Value>(line) {
            if let Some(log_level) = parsed.get("level").and_then(|v| v.as_str()) {
                return log_level.to_lowercase() == level.to_lowercase();
            }
        }
        
        // Fallback to string search
        line.to_lowercase().contains(&level.to_lowercase())
    }

    /// Check if a log line matches the date range
    fn line_matches_date_range(&self, line: &str) -> bool {
        // This is a simplified implementation
        // A full implementation would parse timestamps and compare dates
        
        if let Some(ref since) = self.since {
            if !line.contains(since) {
                // Simplified check - would need proper date parsing
                return true; // Allow for now
            }
        }
        
        if let Some(ref until) = self.until {
            if !line.contains(until) {
                // Simplified check - would need proper date parsing
                return true; // Allow for now
            }
        }
        
        true
    }

    /// Print lines with formatting
    fn print_lines(&self, lines: &[String]) {
        for line in lines {
            match self.format {
                LogFormat::Text => {
                    self.print_formatted_line(line);
                }
                LogFormat::Json => {
                    println!("{}", line);
                }
            }
        }
    }

    /// Print a formatted log line
    fn print_formatted_line(&self, line: &str) {
        // Try to parse as JSON and format nicely
        if let Ok(parsed) = serde_json::from_str::<Value>(line) {
            if let Some(timestamp) = parsed.get("time").and_then(|v| v.as_str()) {
                if let Some(level) = parsed.get("level").and_then(|v| v.as_str()) {
                    if let Some(msg) = parsed.get("msg").and_then(|v| v.as_str()) {
                        // Format: [TIME] LEVEL: MESSAGE
                        let time_part = if timestamp.len() > 19 {
                            &timestamp[11..19] // Extract HH:MM:SS
                        } else {
                            timestamp
                        };
                        
                        let level_colored = match level.to_uppercase().as_str() {
                            "ERROR" => format!("\x1b[31m{}\x1b[0m", level), // Red
                            "WARN" => format!("\x1b[33m{}\x1b[0m", level),  // Yellow
                            "INFO" => format!("\x1b[32m{}\x1b[0m", level),  // Green
                            "DEBUG" => format!("\x1b[36m{}\x1b[0m", level), // Cyan
                            _ => level.to_string(),
                        };
                        
                        println!("[{}] {}: {}", time_part, level_colored, msg);
                        return;
                    }
                }
            }
        }
        
        // Fallback to raw line
        println!("{}", line);
    }

    /// Export logs to a file
    async fn export_logs(&self, lines: &[String], export_path: &PathBuf) -> Result<()> {
        let content = lines.join("\n");
        fs::write(export_path, content).await
            .with_context(|| format!("Failed to export logs to: {}", export_path.display()))?;
        Ok(())
    }

    /// Archive old logs
    async fn archive_logs(&self, config: &Config, older_than_days: u32) -> Result<()> {
        let log_file = self.get_log_file_path(config)?;
        
        if !log_file.exists() {
            println!("No log file found to archive.");
            return Ok(());
        }

        let metadata = fs::metadata(&log_file).await?;
        let modified_time = metadata.modified()?;
        let age = std::time::SystemTime::now()
            .duration_since(modified_time)?
            .as_secs() / (24 * 60 * 60); // Convert to days

        if age > older_than_days as u64 {
            let archive_name = format!("goofy-{}.log", 
                chrono::Utc::now().format("%Y%m%d-%H%M%S"));
            let archive_path = log_file.with_file_name(archive_name);
            
            fs::rename(&log_file, &archive_path).await?;
            fs::write(&log_file, "").await?; // Create new empty log file
            
            println!("Archived log file to: {}", archive_path.display());
        } else {
            println!("Log file is not old enough to archive (age: {} days, threshold: {} days)", 
                age, older_than_days);
        }

        Ok(())
    }

    /// Show log statistics
    async fn show_log_stats(&self, config: &Config) -> Result<()> {
        let log_file = self.get_log_file_path(config)?;
        
        if !log_file.exists() {
            println!("No log file found.");
            return Ok(());
        }

        let content = fs::read_to_string(&log_file).await?;
        let lines: Vec<&str> = content.lines().collect();
        
        let mut stats = LogStats::default();
        
        for line in &lines {
            stats.total_lines += 1;
            
            if let Ok(parsed) = serde_json::from_str::<Value>(line) {
                if let Some(level) = parsed.get("level").and_then(|v| v.as_str()) {
                    match level.to_uppercase().as_str() {
                        "DEBUG" => stats.debug_count += 1,
                        "INFO" => stats.info_count += 1,
                        "WARN" => stats.warn_count += 1,
                        "ERROR" => stats.error_count += 1,
                        _ => stats.other_count += 1,
                    }
                }
            }
        }

        let metadata = fs::metadata(&log_file).await?;
        let file_size = metadata.len();
        let modified_time = metadata.modified()?;

        println!("Log Statistics");
        println!("==============");
        println!("File: {}", log_file.display());
        println!("Size: {} bytes ({:.2} KB)", file_size, file_size as f64 / 1024.0);
        println!("Last modified: {}", humantime::format_rfc3339_seconds(modified_time.into()));
        println!("Total lines: {}", stats.total_lines);
        println!("  DEBUG: {}", stats.debug_count);
        println!("  INFO:  {}", stats.info_count);
        println!("  WARN:  {}", stats.warn_count);
        println!("  ERROR: {}", stats.error_count);
        println!("  OTHER: {}", stats.other_count);

        Ok(())
    }
}

/// Log statistics structure
#[derive(Default)]
struct LogStats {
    total_lines: usize,
    debug_count: usize,
    info_count: usize,
    warn_count: usize,
    error_count: usize,
    other_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_read_tail_lines() {
        let dir = tempdir().unwrap();
        let log_file = dir.path().join("test.log");
        
        // Create a test log file
        let content = "line1\nline2\nline3\nline4\nline5\n";
        fs::write(&log_file, content).await.unwrap();
        
        let cmd = LogsCommand {
            follow: false,
            tail: 3,
            level: None,
            export: None,
            since: None,
            until: None,
            format: LogFormat::Text,
            command: None,
        };
        
        let file = File::open(&log_file).unwrap();
        let lines = cmd.read_tail_lines(file).unwrap();
        
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line3");
        assert_eq!(lines[1], "line4");
        assert_eq!(lines[2], "line5");
    }

    #[test]
    fn test_line_matches_level() {
        let cmd = LogsCommand {
            follow: false,
            tail: 100,
            level: None,
            export: None,
            since: None,
            until: None,
            format: LogFormat::Text,
            command: None,
        };
        
        let json_line = r#"{"level":"ERROR","msg":"test error","time":"2024-01-01T12:00:00Z"}"#;
        assert!(cmd.line_matches_level(json_line, "error"));
        assert!(!cmd.line_matches_level(json_line, "info"));
        
        let plain_line = "ERROR: Something went wrong";
        assert!(cmd.line_matches_level(plain_line, "error"));
        assert!(!cmd.line_matches_level(plain_line, "info"));
    }
}