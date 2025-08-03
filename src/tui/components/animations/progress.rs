//! Progress bars and indicators for showing completion status.
//! 
//! This module provides various progress indicator components with smooth
//! animations and customizable styling for displaying operation progress.

use super::animation_engine::{AnimationEngine, AnimationConfig, EasingType};
use super::interpolation::{RgbColor, ColorGradient, Interpolatable};
use anyhow::Result;
use ratatui::style::{Color, Style};
use ratatui::text::{Span, Line};
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::layout::Rect;
use std::time::Duration;

/// Progress bar style variants
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressStyle {
    /// Simple filled bar
    Bar,
    /// Blocks that fill up
    Blocks,
    /// Dots that appear
    Dots,
    /// Gradient fill
    Gradient,
    /// Pulse wave effect
    Pulse,
    /// Circular progress
    Circle,
    /// ASCII art style
    Ascii,
}

/// Progress indicator configuration
#[derive(Debug, Clone)]
pub struct ProgressConfig {
    pub style: ProgressStyle,
    pub width: usize,
    pub height: usize,
    pub show_percentage: bool,
    pub show_label: bool,
    pub label: String,
    pub foreground_color: RgbColor,
    pub background_color: RgbColor,
    pub border_color: Option<RgbColor>,
    pub animate_transitions: bool,
    pub transition_duration: Duration,
}

impl Default for ProgressConfig {
    fn default() -> Self {
        Self {
            style: ProgressStyle::Bar,
            width: 20,
            height: 1,
            show_percentage: true,
            show_label: false,
            label: String::new(),
            foreground_color: RgbColor::new(100, 200, 100),
            background_color: RgbColor::new(50, 50, 50),
            border_color: None,
            animate_transitions: true,
            transition_duration: Duration::from_millis(200),
        }
    }
}

impl ProgressConfig {
    pub fn new(style: ProgressStyle) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width.max(1);
        self
    }

    pub fn with_height(mut self, height: usize) -> Self {
        self.height = height.max(1);
        self
    }

    pub fn with_label(mut self, label: String) -> Self {
        self.label = label;
        self.show_label = true;
        self
    }

    pub fn with_colors(mut self, foreground: RgbColor, background: RgbColor) -> Self {
        self.foreground_color = foreground;
        self.background_color = background;
        self
    }

    pub fn with_border(mut self, color: RgbColor) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    pub fn with_animation(mut self, duration: Duration) -> Self {
        self.animate_transitions = true;
        self.transition_duration = duration;
        self
    }

    pub fn no_animation(mut self) -> Self {
        self.animate_transitions = false;
        self
    }

    /// Quick configurations for common use cases
    pub fn file_download() -> Self {
        Self::new(ProgressStyle::Bar)
            .with_width(30)
            .with_label("Downloading".to_string())
            .with_colors(
                RgbColor::new(100, 200, 255),
                RgbColor::new(30, 30, 60),
            )
    }

    pub fn loading_data() -> Self {
        Self::new(ProgressStyle::Blocks)
            .with_width(20)
            .with_label("Loading".to_string())
            .with_colors(
                RgbColor::new(255, 200, 100),
                RgbColor::new(60, 40, 20),
            )
    }

    pub fn processing() -> Self {
        Self::new(ProgressStyle::Gradient)
            .with_width(25)
            .with_label("Processing".to_string())
            .with_colors(
                RgbColor::new(200, 100, 255),
                RgbColor::new(40, 20, 60),
            )
    }
}

/// Animated progress indicator
#[derive(Debug)]
pub struct ProgressIndicator {
    config: ProgressConfig,
    current_progress: f32,
    target_progress: f32,
    animation: Option<AnimationEngine>,
    gradient: Option<ColorGradient>,
    start_progress: f32,
}

impl ProgressIndicator {
    pub fn new(config: ProgressConfig) -> Self {
        // Create gradient for gradient style
        let gradient = if config.style == ProgressStyle::Gradient {
            Some(
                ColorGradient::new()
                    .add_stop(0.0, config.background_color)
                    .add_stop(1.0, config.foreground_color)
            )
        } else {
            None
        };

        Self {
            config,
            current_progress: 0.0,
            target_progress: 0.0,
            animation: None,
            gradient,
            start_progress: 0.0,
        }
    }

    /// Set the progress value (0.0 to 1.0)
    pub fn set_progress(&mut self, progress: f32) {
        let progress = progress.clamp(0.0, 1.0);
        self.target_progress = progress;

        if self.config.animate_transitions {
            self.start_progress = self.current_progress;
            let animation_config = AnimationConfig::new(self.config.transition_duration)
                .with_easing(EasingType::EaseOutQuad);
            self.animation = Some(AnimationEngine::new(animation_config));
            
            if let Some(animation) = &mut self.animation {
                animation.start();
            }
        } else {
            self.current_progress = progress;
        }
    }

    /// Update the animation
    pub fn update(&mut self) -> Result<bool> {
        if let Some(animation) = &mut self.animation {
            if animation.should_update() {
                let eased_progress = animation.eased_progress();
                self.current_progress = self.start_progress.interpolate(&self.target_progress, eased_progress);
                Ok(true)
            } else if animation.is_completed() {
                self.current_progress = self.target_progress;
                self.animation = None;
                Ok(false)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Get current progress value
    pub fn progress(&self) -> f32 {
        self.current_progress
    }

    /// Render the progress indicator
    pub fn render(&self) -> Vec<Line> {
        let mut lines = Vec::new();

        // Add label if configured
        if self.config.show_label && !self.config.label.is_empty() {
            let label_line = Line::from(vec![
                Span::styled(
                    &self.config.label,
                    Style::default().fg(self.config.foreground_color.to_color()),
                ),
            ]);
            lines.push(label_line);
        }

        // Render progress bar based on style
        match self.config.style {
            ProgressStyle::Bar => lines.extend(self.render_bar()),
            ProgressStyle::Blocks => lines.extend(self.render_blocks()),
            ProgressStyle::Dots => lines.extend(self.render_dots()),
            ProgressStyle::Gradient => lines.extend(self.render_gradient()),
            ProgressStyle::Pulse => lines.extend(self.render_pulse()),
            ProgressStyle::Circle => lines.extend(self.render_circle()),
            ProgressStyle::Ascii => lines.extend(self.render_ascii()),
        }

        // Add percentage if configured
        if self.config.show_percentage {
            let percentage = (self.current_progress * 100.0) as u8;
            let percentage_line = Line::from(vec![
                Span::styled(
                    format!("{}%", percentage),
                    Style::default().fg(self.config.foreground_color.to_color()),
                ),
            ]);
            lines.push(percentage_line);
        }

        lines
    }

    /// Render simple bar style
    fn render_bar(&self) -> Vec<Line> {
        let filled_width = (self.current_progress * self.config.width as f32) as usize;
        let empty_width = self.config.width - filled_width;

        let mut spans = Vec::new();

        // Add border if configured
        if self.config.border_color.is_some() {
            spans.push(Span::styled(
                "[",
                Style::default().fg(self.config.border_color.unwrap().to_color()),
            ));
        }

        // Filled portion
        if filled_width > 0 {
            spans.push(Span::styled(
                "█".repeat(filled_width),
                Style::default().fg(self.config.foreground_color.to_color()),
            ));
        }

        // Empty portion
        if empty_width > 0 {
            spans.push(Span::styled(
                "░".repeat(empty_width),
                Style::default().fg(self.config.background_color.to_color()),
            ));
        }

        // Closing border
        if self.config.border_color.is_some() {
            spans.push(Span::styled(
                "]",
                Style::default().fg(self.config.border_color.unwrap().to_color()),
            ));
        }

        vec![Line::from(spans)]
    }

    /// Render blocks style
    fn render_blocks(&self) -> Vec<Line> {
        let filled_blocks = (self.current_progress * self.config.width as f32) as usize;
        let mut spans = Vec::new();

        for i in 0..self.config.width {
            let block_char = if i < filled_blocks {
                "▓"
            } else {
                "░"
            };

            let color = if i < filled_blocks {
                self.config.foreground_color
            } else {
                self.config.background_color
            };

            spans.push(Span::styled(
                block_char,
                Style::default().fg(color.to_color()),
            ));
        }

        vec![Line::from(spans)]
    }

    /// Render dots style
    fn render_dots(&self) -> Vec<Line> {
        let filled_dots = (self.current_progress * self.config.width as f32) as usize;
        let mut spans = Vec::new();

        for i in 0..self.config.width {
            let dot_char = if i < filled_dots {
                "●"
            } else {
                "○"
            };

            let color = if i < filled_dots {
                self.config.foreground_color
            } else {
                self.config.background_color
            };

            spans.push(Span::styled(
                dot_char,
                Style::default().fg(color.to_color()),
            ));
        }

        vec![Line::from(spans)]
    }

    /// Render gradient style
    fn render_gradient(&self) -> Vec<Line> {
        let mut spans = Vec::new();
        
        if let Some(gradient) = &self.gradient {
            let filled_width = (self.current_progress * self.config.width as f32) as usize;
            
            for i in 0..self.config.width {
                let char = if i < filled_width {
                    "█"
                } else {
                    "░"
                };

                let color = if i < filled_width {
                    let gradient_pos = i as f32 / self.config.width as f32;
                    gradient.evaluate(gradient_pos)
                } else {
                    self.config.background_color
                };

                spans.push(Span::styled(
                    char,
                    Style::default().fg(color.to_color()),
                ));
            }
        } else {
            // Fallback to bar style
            return self.render_bar();
        }

        vec![Line::from(spans)]
    }

    /// Render pulse style
    fn render_pulse(&self) -> Vec<Line> {
        let filled_width = (self.current_progress * self.config.width as f32) as usize;
        let mut spans = Vec::new();

        // Create pulsing effect
        let pulse_intensity = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f32 / 500.0)
            .sin().abs();

        for i in 0..self.config.width {
            let char = if i < filled_width {
                "█"
            } else {
                "░"
            };

            let color = if i < filled_width {
                let r = (self.config.foreground_color.r as f32 * (0.5 + 0.5 * pulse_intensity)) as u8;
                let g = (self.config.foreground_color.g as f32 * (0.5 + 0.5 * pulse_intensity)) as u8;
                let b = (self.config.foreground_color.b as f32 * (0.5 + 0.5 * pulse_intensity)) as u8;
                RgbColor::new(r, g, b)
            } else {
                self.config.background_color
            };

            spans.push(Span::styled(
                char,
                Style::default().fg(color.to_color()),
            ));
        }

        vec![Line::from(spans)]
    }

    /// Render circular progress (ASCII approximation)
    fn render_circle(&self) -> Vec<Line> {
        let steps = 8;
        let current_step = (self.current_progress * steps as f32) as usize;
        
        let circle_chars = ["○", "◔", "◑", "◕", "●", "●", "●", "●"];
        let char = circle_chars[current_step.min(circle_chars.len() - 1)];
        
        vec![Line::from(vec![
            Span::styled(
                char,
                Style::default().fg(self.config.foreground_color.to_color()),
            ),
        ])]
    }

    /// Render ASCII art style
    fn render_ascii(&self) -> Vec<Line> {
        let filled_width = (self.current_progress * self.config.width as f32) as usize;
        let mut spans = Vec::new();

        spans.push(Span::raw("["));
        
        for i in 0..self.config.width {
            let char = if i < filled_width {
                "="
            } else if i == filled_width && filled_width < self.config.width {
                ">"
            } else {
                " "
            };

            spans.push(Span::styled(
                char,
                Style::default().fg(if i <= filled_width {
                    self.config.foreground_color.to_color()
                } else {
                    self.config.background_color.to_color()
                }),
            ));
        }

        spans.push(Span::raw("]"));
        vec![Line::from(spans)]
    }

    /// Check if progress is animating
    pub fn is_animating(&self) -> bool {
        self.animation.is_some()
    }

    /// Get the estimated width in characters
    pub fn estimated_width(&self) -> usize {
        let mut width = self.config.width;
        
        // Add borders if present
        if self.config.border_color.is_some() {
            width += 2; // [ and ]
        }
        
        width
    }

    /// Get the estimated height in lines
    pub fn estimated_height(&self) -> usize {
        let mut height = 1; // Progress bar itself
        
        if self.config.show_label && !self.config.label.is_empty() {
            height += 1;
        }
        
        if self.config.show_percentage {
            height += 1;
        }
        
        height
    }
}

/// Collection of progress indicator presets
pub struct ProgressPresets;

impl ProgressPresets {
    /// File operation progress
    pub fn file_operation() -> ProgressIndicator {
        ProgressIndicator::new(ProgressConfig::file_download())
    }

    /// Data loading progress
    pub fn data_loading() -> ProgressIndicator {
        ProgressIndicator::new(ProgressConfig::loading_data())
    }

    /// Processing progress
    pub fn processing() -> ProgressIndicator {
        ProgressIndicator::new(ProgressConfig::processing())
    }

    /// Simple progress bar
    pub fn simple() -> ProgressIndicator {
        ProgressIndicator::new(ProgressConfig::default())
    }

    /// Circular progress indicator
    pub fn circular() -> ProgressIndicator {
        ProgressIndicator::new(
            ProgressConfig::new(ProgressStyle::Circle)
                .with_colors(
                    RgbColor::new(100, 200, 100),
                    RgbColor::new(50, 50, 50),
                )
        )
    }

    /// Minimal ASCII progress
    pub fn minimal() -> ProgressIndicator {
        ProgressIndicator::new(
            ProgressConfig::new(ProgressStyle::Ascii)
                .with_width(15)
                .show_percentage(false)
                .no_animation()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let config = ProgressConfig::new(ProgressStyle::Bar)
            .with_width(20)
            .with_label("Test".to_string());
        
        let progress = ProgressIndicator::new(config);
        assert_eq!(progress.progress(), 0.0);
        assert_eq!(progress.estimated_width(), 20);
    }

    #[test]
    fn test_progress_setting() {
        let config = ProgressConfig::default().no_animation();
        let mut progress = ProgressIndicator::new(config);
        
        progress.set_progress(0.5);
        assert_eq!(progress.progress(), 0.5);
        
        progress.set_progress(1.2); // Should clamp to 1.0
        assert_eq!(progress.progress(), 1.0);
        
        progress.set_progress(-0.1); // Should clamp to 0.0
        assert_eq!(progress.progress(), 0.0);
    }

    #[test]
    fn test_progress_animation() {
        let config = ProgressConfig::default()
            .with_animation(Duration::from_millis(100));
        let mut progress = ProgressIndicator::new(config);
        
        progress.set_progress(0.5);
        assert!(progress.is_animating());
        
        // Progress should be between 0.0 and 0.5 during animation
        let current = progress.progress();
        assert!(current >= 0.0 && current <= 0.5);
    }

    #[test]
    fn test_progress_presets() {
        let file_progress = ProgressPresets::file_operation();
        let data_progress = ProgressPresets::data_loading();
        let simple_progress = ProgressPresets::simple();
        
        // Just verify they can be created without panicking
        assert_eq!(file_progress.progress(), 0.0);
        assert_eq!(data_progress.progress(), 0.0);
        assert_eq!(simple_progress.progress(), 0.0);
    }

    #[test]
    fn test_estimated_dimensions() {
        let config = ProgressConfig::new(ProgressStyle::Bar)
            .with_width(20)
            .with_label("Test".to_string())
            .show_percentage(true)
            .with_border(RgbColor::new(255, 255, 255));
        
        let progress = ProgressIndicator::new(config);
        assert_eq!(progress.estimated_width(), 22); // 20 + 2 for borders
        assert_eq!(progress.estimated_height(), 3); // bar + label + percentage
    }
}