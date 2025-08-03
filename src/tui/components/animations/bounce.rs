//! Bounce effects for notifications and interactive feedback.
//! 
//! This module provides various bounce animation styles for drawing attention
//! to UI elements, providing feedback for user interactions, and creating
//! playful visual effects.

use super::animation_engine::{AnimationEngine, AnimationConfig, EasingType};
use super::interpolation::{RgbColor, Interpolatable};
use anyhow::Result;
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Span, Line};
use ratatui::layout::Rect;
use std::time::Duration;

/// Bounce animation styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BounceStyle {
    /// Simple vertical bounce
    Vertical,
    /// Simple horizontal bounce
    Horizontal,
    /// Scale bounce (growing/shrinking)
    Scale,
    /// Elastic bounce with overshoot
    Elastic,
    /// Gentle pulse bounce
    Pulse,
    /// Rubber band snap effect
    RubberBand,
    /// Jello wobble effect
    Jello,
    /// Heartbeat bounce
    Heartbeat,
    /// Shake animation
    Shake,
}

/// Bounce animation configuration
#[derive(Debug, Clone)]
pub struct BounceConfig {
    pub style: BounceStyle,
    pub duration: Duration,
    pub intensity: f32, // 0.0 to 1.0, controls bounce magnitude
    pub bounces: u32,   // Number of bounces
    pub damping: f32,   // Damping factor for each bounce (0.0 to 1.0)
    pub delay: Duration,
    pub loop_count: Option<u32>, // None = run once
}

impl Default for BounceConfig {
    fn default() -> Self {
        Self {
            style: BounceStyle::Vertical,
            duration: Duration::from_millis(600),
            intensity: 1.0,
            bounces: 3,
            damping: 0.7,
            delay: Duration::from_millis(0),
            loop_count: Some(1),
        }
    }
}

impl BounceConfig {
    pub fn new(style: BounceStyle) -> Self {
        Self {
            style,
            ..Default::default()
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 2.0);
        self
    }

    pub fn with_bounces(mut self, bounces: u32) -> Self {
        self.bounces = bounces.max(1);
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping.clamp(0.0, 1.0);
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn with_loop_count(mut self, count: u32) -> Self {
        self.loop_count = Some(count);
        self
    }

    pub fn infinite(mut self) -> Self {
        self.loop_count = None;
        self
    }

    /// Quick configurations for common scenarios
    pub fn notification() -> Self {
        Self::new(BounceStyle::Scale)
            .with_duration(Duration::from_millis(400))
            .with_intensity(0.3)
            .with_bounces(2)
    }

    pub fn button_press() -> Self {
        Self::new(BounceStyle::Scale)
            .with_duration(Duration::from_millis(150))
            .with_intensity(0.15)
            .with_bounces(1)
    }

    pub fn error_shake() -> Self {
        Self::new(BounceStyle::Shake)
            .with_duration(Duration::from_millis(300))
            .with_intensity(0.8)
            .with_bounces(4)
    }

    pub fn success_bounce() -> Self {
        Self::new(BounceStyle::Vertical)
            .with_duration(Duration::from_millis(500))
            .with_intensity(0.6)
            .with_bounces(3)
            .with_damping(0.6)
    }

    pub fn attention_pulse() -> Self {
        Self::new(BounceStyle::Pulse)
            .with_duration(Duration::from_millis(800))
            .with_intensity(0.4)
            .infinite()
    }

    pub fn elastic_entrance() -> Self {
        Self::new(BounceStyle::Elastic)
            .with_duration(Duration::from_millis(600))
            .with_intensity(1.2)
            .with_bounces(2)
    }

    pub fn rubber_band() -> Self {
        Self::new(BounceStyle::RubberBand)
            .with_duration(Duration::from_millis(500))
            .with_intensity(0.8)
    }

    pub fn jello_wobble() -> Self {
        Self::new(BounceStyle::Jello)
            .with_duration(Duration::from_millis(600))
            .with_intensity(0.6)
    }
}

/// Bounce effect state
#[derive(Debug, Clone)]
pub struct BounceState {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub intensity_multiplier: f32,
}

impl Default for BounceState {
    fn default() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            intensity_multiplier: 1.0,
        }
    }
}

/// Bounce animation component
#[derive(Debug)]
pub struct BounceAnimation {
    config: BounceConfig,
    animation: AnimationEngine,
    current_state: BounceState,
    is_active: bool,
    content: Vec<Line<'static>>,
    base_area: Rect,
}

impl BounceAnimation {
    pub fn new(config: BounceConfig) -> Self {
        let easing = match config.style {
            BounceStyle::Vertical | BounceStyle::Horizontal => EasingType::EaseOutBounce,
            BounceStyle::Scale => EasingType::EaseOutElastic,
            BounceStyle::Elastic => EasingType::EaseOutElastic,
            BounceStyle::Pulse => EasingType::EaseInOut,
            BounceStyle::RubberBand => EasingType::EaseInOutBack,
            BounceStyle::Jello => EasingType::EaseInOutElastic,
            BounceStyle::Heartbeat => EasingType::EaseOutBounce,
            BounceStyle::Shake => EasingType::Linear,
        };

        let animation_config = AnimationConfig::new(config.duration)
            .with_easing(easing)
            .with_delay(config.delay);

        let animation_config = if let Some(count) = config.loop_count {
            animation_config.with_loop_count(count)
        } else {
            animation_config.infinite()
        };

        Self {
            config,
            animation: AnimationEngine::new(animation_config),
            current_state: BounceState::default(),
            is_active: false,
            content: Vec::new(),
            base_area: Rect::default(),
        }
    }

    /// Set the content to be bounced
    pub fn set_content(&mut self, content: Vec<Line<'static>>) {
        self.content = content;
    }

    /// Set content from a single string
    pub fn set_text(&mut self, text: String) {
        self.content = vec![Line::from(text)];
    }

    /// Set the base area for the bounce effect
    pub fn set_area(&mut self, area: Rect) {
        self.base_area = area;
    }

    /// Start the bounce animation
    pub fn start(&mut self) {
        self.animation.start();
        self.is_active = true;
    }

    /// Stop the bounce animation
    pub fn stop(&mut self) {
        self.animation.stop();
        self.is_active = false;
        self.current_state = BounceState::default();
    }

    /// Trigger a one-shot bounce (restarts if already running)
    pub fn trigger(&mut self) {
        self.stop();
        self.start();
    }

    /// Update the bounce animation
    pub fn update(&mut self) -> Result<bool> {
        if !self.is_active {
            return Ok(false);
        }

        if self.animation.should_update() {
            let progress = self.animation.eased_progress();
            self.calculate_bounce_state(progress);
            Ok(true)
        } else if self.animation.is_completed() {
            if self.config.loop_count.is_some() {
                self.is_active = false;
                self.current_state = BounceState::default();
            }
            Ok(false)
        } else {
            Ok(false)
        }
    }

    /// Calculate the current bounce state based on progress
    fn calculate_bounce_state(&mut self, progress: f32) {
        match self.config.style {
            BounceStyle::Vertical => self.calculate_vertical_bounce(progress),
            BounceStyle::Horizontal => self.calculate_horizontal_bounce(progress),
            BounceStyle::Scale => self.calculate_scale_bounce(progress),
            BounceStyle::Elastic => self.calculate_elastic_bounce(progress),
            BounceStyle::Pulse => self.calculate_pulse_bounce(progress),
            BounceStyle::RubberBand => self.calculate_rubber_band(progress),
            BounceStyle::Jello => self.calculate_jello(progress),
            BounceStyle::Heartbeat => self.calculate_heartbeat(progress),
            BounceStyle::Shake => self.calculate_shake(progress),
        }
    }

    /// Calculate vertical bounce effect
    fn calculate_vertical_bounce(&mut self, progress: f32) {
        let bounce_cycle = progress * self.config.bounces as f32;
        let cycle_phase = bounce_cycle.fract();
        let bounce_index = bounce_cycle.floor() as u32;
        
        // Calculate damping for this bounce
        let damping = self.config.damping.powi(bounce_index as i32);
        
        // Sine wave for bounce motion
        let bounce_value = (cycle_phase * std::f32::consts::PI).sin();
        
        self.current_state.offset_y = -bounce_value * self.config.intensity * 10.0 * damping;
        self.current_state.offset_x = 0.0;
        self.current_state.scale_x = 1.0;
        self.current_state.scale_y = 1.0 + bounce_value * 0.1 * self.config.intensity * damping;
    }

    /// Calculate horizontal bounce effect
    fn calculate_horizontal_bounce(&mut self, progress: f32) {
        let bounce_cycle = progress * self.config.bounces as f32;
        let cycle_phase = bounce_cycle.fract();
        let bounce_index = bounce_cycle.floor() as u32;
        
        let damping = self.config.damping.powi(bounce_index as i32);
        let bounce_value = (cycle_phase * std::f32::consts::PI).sin();
        
        self.current_state.offset_x = bounce_value * self.config.intensity * 5.0 * damping;
        self.current_state.offset_y = 0.0;
        self.current_state.scale_x = 1.0 + bounce_value * 0.1 * self.config.intensity * damping;
        self.current_state.scale_y = 1.0;
    }

    /// Calculate scale bounce effect
    fn calculate_scale_bounce(&mut self, progress: f32) {
        let bounce_cycle = progress * self.config.bounces as f32;
        let cycle_phase = bounce_cycle.fract();
        let bounce_index = bounce_cycle.floor() as u32;
        
        let damping = self.config.damping.powi(bounce_index as i32);
        let bounce_value = (cycle_phase * std::f32::consts::PI).sin();
        
        let scale_factor = 1.0 + bounce_value * self.config.intensity * 0.3 * damping;
        
        self.current_state.scale_x = scale_factor;
        self.current_state.scale_y = scale_factor;
        self.current_state.offset_x = 0.0;
        self.current_state.offset_y = 0.0;
    }

    /// Calculate elastic bounce effect
    fn calculate_elastic_bounce(&mut self, progress: f32) {
        let elastic_progress = if progress < 0.5 {
            progress * 2.0
        } else {
            2.0 - progress * 2.0
        };
        
        let elastic_value = (elastic_progress * 10.0).sin() * (1.0 - elastic_progress).powi(3);
        
        self.current_state.scale_x = 1.0 + elastic_value * self.config.intensity * 0.5;
        self.current_state.scale_y = 1.0 + elastic_value * self.config.intensity * 0.5;
        self.current_state.offset_x = elastic_value * self.config.intensity * 3.0;
        self.current_state.offset_y = -elastic_value * self.config.intensity * 3.0;
    }

    /// Calculate pulse bounce effect
    fn calculate_pulse_bounce(&mut self, progress: f32) {
        let pulse_value = (progress * 2.0 * std::f32::consts::PI).sin().abs();
        let scale_factor = 1.0 + pulse_value * self.config.intensity * 0.2;
        
        self.current_state.scale_x = scale_factor;
        self.current_state.scale_y = scale_factor;
        self.current_state.intensity_multiplier = 0.7 + pulse_value * 0.3;
        self.current_state.offset_x = 0.0;
        self.current_state.offset_y = 0.0;
    }

    /// Calculate rubber band effect
    fn calculate_rubber_band(&mut self, progress: f32) {
        let stretch_phase = if progress < 0.3 {
            progress / 0.3
        } else if progress < 0.7 {
            1.0 - (progress - 0.3) / 0.4 * 1.5
        } else {
            -0.5 + (progress - 0.7) / 0.3 * 1.5
        };
        
        self.current_state.scale_x = 1.0 + stretch_phase * self.config.intensity * 0.4;
        self.current_state.scale_y = 1.0 - stretch_phase * self.config.intensity * 0.2;
        self.current_state.offset_x = 0.0;
        self.current_state.offset_y = 0.0;
    }

    /// Calculate jello wobble effect
    fn calculate_jello(&mut self, progress: f32) {
        let wobble_value = (progress * 8.0 * std::f32::consts::PI).sin() * (1.0 - progress);
        
        self.current_state.scale_x = 1.0 + wobble_value * self.config.intensity * 0.1;
        self.current_state.scale_y = 1.0 - wobble_value * self.config.intensity * 0.1;
        self.current_state.rotation = wobble_value * self.config.intensity * 5.0;
        self.current_state.offset_x = 0.0;
        self.current_state.offset_y = 0.0;
    }

    /// Calculate heartbeat effect
    fn calculate_heartbeat(&mut self, progress: f32) {
        let beat_value = if progress < 0.2 {
            (progress / 0.2 * std::f32::consts::PI).sin()
        } else if progress < 0.4 {
            ((progress - 0.2) / 0.2 * std::f32::consts::PI).sin() * 0.7
        } else {
            0.0
        };
        
        let scale_factor = 1.0 + beat_value * self.config.intensity * 0.3;
        
        self.current_state.scale_x = scale_factor;
        self.current_state.scale_y = scale_factor;
        self.current_state.intensity_multiplier = 0.8 + beat_value * 0.2;
        self.current_state.offset_x = 0.0;
        self.current_state.offset_y = 0.0;
    }

    /// Calculate shake effect
    fn calculate_shake(&mut self, progress: f32) {
        let shake_frequency = 20.0;
        let shake_value = (progress * shake_frequency * std::f32::consts::PI).sin();
        let decay = 1.0 - progress;
        
        self.current_state.offset_x = shake_value * self.config.intensity * 3.0 * decay;
        self.current_state.offset_y = (shake_value * 0.3) * self.config.intensity * 2.0 * decay;
        self.current_state.scale_x = 1.0;
        self.current_state.scale_y = 1.0;
    }

    /// Render the bouncing content
    pub fn render(&self) -> Vec<Line> {
        if self.content.is_empty() {
            return Vec::new();
        }

        // Apply bounce effects to the content styling
        self.content
            .iter()
            .map(|line| {
                let spans: Vec<Span> = line
                    .spans
                    .iter()
                    .map(|span| {
                        let mut style = span.style;
                        
                        // Apply intensity effect for pulse and heartbeat
                        if matches!(self.config.style, BounceStyle::Pulse | BounceStyle::Heartbeat) {
                            if let Some(fg) = style.fg {
                                let adjusted_color = self.adjust_color_intensity(fg);
                                style.fg = Some(adjusted_color);
                            }
                        }
                        
                        // Apply scale effect through modifiers (limited in terminal)
                        if self.current_state.scale_x > 1.1 || self.current_state.scale_y > 1.1 {
                            style = style.add_modifier(Modifier::BOLD);
                        }
                        
                        // Apply rotation effect through underlining (visual approximation)
                        if self.current_state.rotation.abs() > 2.0 {
                            style = style.add_modifier(Modifier::UNDERLINED);
                        }
                        
                        Span::styled(span.content.clone(), style)
                    })
                    .collect();
                Line::from(spans)
            })
            .collect()
    }

    /// Adjust color intensity based on bounce state
    fn adjust_color_intensity(&self, color: Color) -> Color {
        if let Color::Rgb(r, g, b) = color {
            let intensity = self.current_state.intensity_multiplier;
            let new_r = (r as f32 * intensity).min(255.0) as u8;
            let new_g = (g as f32 * intensity).min(255.0) as u8;
            let new_b = (b as f32 * intensity).min(255.0) as u8;
            Color::Rgb(new_r, new_g, new_b)
        } else {
            color
        }
    }

    /// Get the current bounce area (base area with offsets applied)
    pub fn current_area(&self) -> Rect {
        Rect {
            x: (self.base_area.x as f32 + self.current_state.offset_x).max(0.0) as u16,
            y: (self.base_area.y as f32 + self.current_state.offset_y).max(0.0) as u16,
            width: (self.base_area.width as f32 * self.current_state.scale_x).max(1.0) as u16,
            height: (self.base_area.height as f32 * self.current_state.scale_y).max(1.0) as u16,
        }
    }

    /// Get the current bounce state
    pub fn state(&self) -> &BounceState {
        &self.current_state
    }

    /// Check if animation is running
    pub fn is_running(&self) -> bool {
        self.is_active && self.animation.is_running()
    }

    /// Check if animation is completed
    pub fn is_completed(&self) -> bool {
        self.animation.is_completed()
    }

    /// Get animation progress
    pub fn progress(&self) -> f32 {
        self.animation.progress()
    }
}

/// Collection of bounce presets for common UI scenarios
pub struct BouncePresets;

impl BouncePresets {
    /// Notification bounce
    pub fn notification() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::notification())
    }

    /// Button press feedback
    pub fn button_press() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::button_press())
    }

    /// Error shake
    pub fn error_shake() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::error_shake())
    }

    /// Success bounce
    pub fn success() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::success_bounce())
    }

    /// Attention-grabbing pulse
    pub fn attention() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::attention_pulse())
    }

    /// Elastic entrance
    pub fn elastic_entrance() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::elastic_entrance())
    }

    /// Rubber band effect
    pub fn rubber_band() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::rubber_band())
    }

    /// Jello wobble
    pub fn jello() -> BounceAnimation {
        BounceAnimation::new(BounceConfig::jello_wobble())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounce_config_creation() {
        let config = BounceConfig::new(BounceStyle::Scale)
            .with_duration(Duration::from_millis(500))
            .with_intensity(0.8)
            .with_bounces(3)
            .with_damping(0.6);
        
        assert_eq!(config.style, BounceStyle::Scale);
        assert_eq!(config.duration, Duration::from_millis(500));
        assert_eq!(config.intensity, 0.8);
        assert_eq!(config.bounces, 3);
        assert_eq!(config.damping, 0.6);
    }

    #[test]
    fn test_bounce_animation_lifecycle() {
        let config = BounceConfig::notification();
        let mut bounce = BounceAnimation::new(config);
        
        assert!(!bounce.is_running());
        
        bounce.start();
        assert!(bounce.is_running());
        
        bounce.stop();
        assert!(!bounce.is_running());
    }

    #[test]
    fn test_bounce_state_default() {
        let state = BounceState::default();
        assert_eq!(state.offset_x, 0.0);
        assert_eq!(state.offset_y, 0.0);
        assert_eq!(state.scale_x, 1.0);
        assert_eq!(state.scale_y, 1.0);
        assert_eq!(state.rotation, 0.0);
        assert_eq!(state.intensity_multiplier, 1.0);
    }

    #[test]
    fn test_trigger_functionality() {
        let config = BounceConfig::button_press();
        let mut bounce = BounceAnimation::new(config);
        
        // Trigger should start the animation
        bounce.trigger();
        assert!(bounce.is_running());
        
        // Trigger again should restart
        bounce.trigger();
        assert!(bounce.is_running());
    }

    #[test]
    fn test_bounce_presets() {
        let notification = BouncePresets::notification();
        let button_press = BouncePresets::button_press();
        let error_shake = BouncePresets::error_shake();
        let success = BouncePresets::success();
        
        // Just verify they can be created without panicking
        assert!(!notification.is_running());
        assert!(!button_press.is_running());
        assert!(!error_shake.is_running());
        assert!(!success.is_running());
    }

    #[test]
    fn test_area_calculation() {
        let config = BounceConfig::new(BounceStyle::Scale);
        let mut bounce = BounceAnimation::new(config);
        
        let base_area = Rect::new(10, 10, 20, 20);
        bounce.set_area(base_area);
        
        // Current area should start as base area
        assert_eq!(bounce.current_area(), base_area);
    }
}