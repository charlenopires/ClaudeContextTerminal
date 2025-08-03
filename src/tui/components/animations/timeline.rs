//! Animation timeline system for sequencing and coordinating multiple animations.
//! 
//! This module provides tools for creating complex animation sequences, including
//! parallel and sequential animations, delays, and synchronization between different
//! animated elements.

use super::animation_engine::{AnimationEngine, AnimationConfig, AnimationState};
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for animations in a timeline
pub type AnimationId = String;

/// Timeline event types
#[derive(Debug, Clone)]
pub enum TimelineEvent {
    AnimationStarted(AnimationId),
    AnimationCompleted(AnimationId),
    AnimationLooped(AnimationId, u32),
    TimelineCompleted,
    TimelinePaused,
    TimelineResumed,
}

/// Animation entry in the timeline
#[derive(Debug)]
pub struct TimelineAnimation {
    pub id: AnimationId,
    pub engine: AnimationEngine,
    pub start_delay: Duration,
    pub depends_on: Vec<AnimationId>,
    pub parallel_group: Option<String>,
    started: bool,
    completed: bool,
}

impl TimelineAnimation {
    pub fn new(id: AnimationId, config: AnimationConfig) -> Self {
        Self {
            id,
            engine: AnimationEngine::new(config),
            start_delay: Duration::from_millis(0),
            depends_on: Vec::new(),
            parallel_group: None,
            started: false,
            completed: false,
        }
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.start_delay = delay;
        self
    }

    pub fn depends_on(mut self, animation_ids: Vec<AnimationId>) -> Self {
        self.depends_on = animation_ids;
        self
    }

    pub fn in_parallel_group(mut self, group: String) -> Self {
        self.parallel_group = Some(group);
        self
    }

    pub fn can_start(&self, completed_animations: &[AnimationId]) -> bool {
        if self.started {
            return false;
        }
        
        // Check if all dependencies are completed
        for dep in &self.depends_on {
            if !completed_animations.contains(dep) {
                return false;
            }
        }
        
        true
    }
}

/// Timeline for orchestrating multiple animations
#[derive(Debug)]
pub struct Timeline {
    animations: HashMap<AnimationId, TimelineAnimation>,
    completed_animations: Vec<AnimationId>,
    start_time: Option<Instant>,
    state: AnimationState,
    events: Vec<TimelineEvent>,
    parallel_groups: HashMap<String, Vec<AnimationId>>,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            completed_animations: Vec::new(),
            start_time: None,
            state: AnimationState::Idle,
            events: Vec::new(),
            parallel_groups: HashMap::new(),
        }
    }

    /// Add an animation to the timeline
    pub fn add_animation(&mut self, animation: TimelineAnimation) -> Result<()> {
        let id = animation.id.clone();
        
        // Register parallel group if specified
        if let Some(group) = &animation.parallel_group {
            self.parallel_groups
                .entry(group.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        self.animations.insert(id, animation);
        Ok(())
    }

    /// Remove an animation from the timeline
    pub fn remove_animation(&mut self, id: &AnimationId) -> Result<()> {
        if let Some(animation) = self.animations.remove(id) {
            // Remove from parallel group if needed
            if let Some(group) = &animation.parallel_group {
                if let Some(group_animations) = self.parallel_groups.get_mut(group) {
                    group_animations.retain(|anim_id| anim_id != id);
                    if group_animations.is_empty() {
                        self.parallel_groups.remove(group);
                    }
                }
            }
        }
        Ok(())
    }

    /// Start the timeline
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.state = AnimationState::Running;
        self.completed_animations.clear();
        self.events.clear();
        
        // Reset all animations
        for animation in self.animations.values_mut() {
            animation.started = false;
            animation.completed = false;
            animation.engine.stop();
        }
    }

    /// Pause the timeline
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
            
            // Pause all running animations
            for animation in self.animations.values_mut() {
                if animation.started && !animation.completed {
                    animation.engine.pause();
                }
            }
            
            self.events.push(TimelineEvent::TimelinePaused);
        }
    }

    /// Resume the timeline
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
            
            // Resume all paused animations
            for animation in self.animations.values_mut() {
                if animation.started && !animation.completed {
                    animation.engine.start(); // This will resume from pause
                }
            }
            
            self.events.push(TimelineEvent::TimelineResumed);
        }
    }

    /// Stop the timeline
    pub fn stop(&mut self) {
        self.state = AnimationState::Idle;
        self.start_time = None;
        self.completed_animations.clear();
        
        // Stop all animations
        for animation in self.animations.values_mut() {
            animation.engine.stop();
            animation.started = false;
            animation.completed = false;
        }
    }

    /// Update the timeline and all animations
    pub fn update(&mut self) -> Result<Vec<TimelineEvent>> {
        let mut new_events = Vec::new();
        
        if self.state != AnimationState::Running {
            return Ok(new_events);
        }
        
        let now = Instant::now();
        let timeline_elapsed = if let Some(start_time) = self.start_time {
            now.duration_since(start_time)
        } else {
            return Ok(new_events);
        };
        
        // Start animations that are ready
        let mut animations_to_start = Vec::new();
        for (id, animation) in &self.animations {
            if animation.can_start(&self.completed_animations) && 
               timeline_elapsed >= animation.start_delay {
                animations_to_start.push(id.clone());
            }
        }
        
        for id in animations_to_start {
            if let Some(animation) = self.animations.get_mut(&id) {
                animation.engine.start();
                animation.started = true;
                new_events.push(TimelineEvent::AnimationStarted(id));
            }
        }
        
        // Update running animations
        let mut completed_this_frame = Vec::new();
        for (id, animation) in &mut self.animations {
            if animation.started && !animation.completed {
                let was_completed = animation.engine.is_completed();
                let was_loop_count = animation.engine.current_loop();
                
                if animation.engine.should_update() {
                    // Animation is still running
                } else if animation.engine.is_completed() && !was_completed {
                    // Animation just completed
                    animation.completed = true;
                    completed_this_frame.push(id.clone());
                    new_events.push(TimelineEvent::AnimationCompleted(id.clone()));
                }
                
                // Check for loop events
                let current_loop = animation.engine.current_loop();
                if current_loop > was_loop_count {
                    new_events.push(TimelineEvent::AnimationLooped(id.clone(), current_loop));
                }
            }
        }
        
        // Add newly completed animations to the list
        self.completed_animations.extend(completed_this_frame);
        
        // Check if timeline is complete
        let all_completed = self.animations.values().all(|anim| anim.completed);
        if all_completed && !self.animations.is_empty() {
            self.state = AnimationState::Completed;
            new_events.push(TimelineEvent::TimelineCompleted);
        }
        
        // Store events for retrieval
        self.events.extend(new_events.clone());
        
        Ok(new_events)
    }

    /// Get the current state of the timeline
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Check if timeline is running
    pub fn is_running(&self) -> bool {
        self.state == AnimationState::Running
    }

    /// Check if timeline is completed
    pub fn is_completed(&self) -> bool {
        self.state == AnimationState::Completed
    }

    /// Get overall timeline progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.animations.is_empty() {
            return 1.0;
        }
        
        let completed_count = self.completed_animations.len();
        let total_count = self.animations.len();
        completed_count as f32 / total_count as f32
    }

    /// Get progress of a specific animation
    pub fn animation_progress(&self, id: &AnimationId) -> Option<f32> {
        self.animations.get(id).map(|anim| anim.engine.progress())
    }

    /// Get eased progress of a specific animation
    pub fn animation_eased_progress(&self, id: &AnimationId) -> Option<f32> {
        self.animations.get(id).map(|anim| anim.engine.eased_progress())
    }

    /// Get all events that occurred during the last update
    pub fn events(&self) -> &[TimelineEvent] {
        &self.events
    }

    /// Clear accumulated events
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Get list of running animations
    pub fn running_animations(&self) -> Vec<&AnimationId> {
        self.animations
            .iter()
            .filter(|(_, anim)| anim.started && !anim.completed)
            .map(|(id, _)| id)
            .collect()
    }

    /// Get list of completed animations
    pub fn completed_animations(&self) -> &[AnimationId] {
        &self.completed_animations
    }

    /// Check if a specific animation is running
    pub fn is_animation_running(&self, id: &AnimationId) -> bool {
        self.animations
            .get(id)
            .map(|anim| anim.started && !anim.completed && anim.engine.is_running())
            .unwrap_or(false)
    }

    /// Get animations in a parallel group
    pub fn parallel_group_animations(&self, group: &str) -> Vec<&AnimationId> {
        self.parallel_groups
            .get(group)
            .map(|ids| ids.iter().collect())
            .unwrap_or_default()
    }

    /// Check if all animations in a parallel group are completed
    pub fn is_parallel_group_completed(&self, group: &str) -> bool {
        if let Some(animation_ids) = self.parallel_groups.get(group) {
            animation_ids.iter().all(|id| self.completed_animations.contains(id))
        } else {
            true // Empty group is considered completed
        }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating complex animation timelines
pub struct TimelineBuilder {
    timeline: Timeline,
}

impl TimelineBuilder {
    pub fn new() -> Self {
        Self {
            timeline: Timeline::new(),
        }
    }

    /// Add a simple animation
    pub fn add(mut self, id: AnimationId, config: AnimationConfig) -> Self {
        let animation = TimelineAnimation::new(id, config);
        let _ = self.timeline.add_animation(animation);
        self
    }

    /// Add an animation with delay
    pub fn add_with_delay(mut self, id: AnimationId, config: AnimationConfig, delay: Duration) -> Self {
        let animation = TimelineAnimation::new(id, config).with_delay(delay);
        let _ = self.timeline.add_animation(animation);
        self
    }

    /// Add an animation that depends on others
    pub fn add_sequential(mut self, id: AnimationId, config: AnimationConfig, depends_on: Vec<AnimationId>) -> Self {
        let animation = TimelineAnimation::new(id, config).depends_on(depends_on);
        let _ = self.timeline.add_animation(animation);
        self
    }

    /// Add an animation to a parallel group
    pub fn add_parallel(mut self, id: AnimationId, config: AnimationConfig, group: String) -> Self {
        let animation = TimelineAnimation::new(id, config).in_parallel_group(group);
        let _ = self.timeline.add_animation(animation);
        self
    }

    /// Add a sequence of animations
    pub fn add_sequence(mut self, animations: Vec<(AnimationId, AnimationConfig)>) -> Self {
        let mut previous_id: Option<AnimationId> = None;
        
        for (id, config) in animations {
            let mut animation = TimelineAnimation::new(id.clone(), config);
            
            if let Some(prev) = previous_id {
                animation = animation.depends_on(vec![prev]);
            }
            
            let _ = self.timeline.add_animation(animation);
            previous_id = Some(id);
        }
        
        self
    }

    /// Add a group of parallel animations
    pub fn add_parallel_group(mut self, group_name: String, animations: Vec<(AnimationId, AnimationConfig)>) -> Self {
        for (id, config) in animations {
            let animation = TimelineAnimation::new(id, config).in_parallel_group(group_name.clone());
            let _ = self.timeline.add_animation(animation);
        }
        self
    }

    /// Build the timeline
    pub fn build(self) -> Timeline {
        self.timeline
    }
}

impl Default for TimelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_timeline_creation() {
        let timeline = Timeline::new();
        assert_eq!(timeline.state(), AnimationState::Idle);
        assert_eq!(timeline.progress(), 1.0); // Empty timeline is complete
    }

    #[test]
    fn test_timeline_builder() {
        let timeline = TimelineBuilder::new()
            .add("fade_in".to_string(), AnimationConfig::fade_in())
            .add_with_delay("slide_in".to_string(), AnimationConfig::slide_in(), Duration::from_millis(100))
            .build();
        
        assert_eq!(timeline.animations.len(), 2);
    }

    #[test]
    fn test_animation_dependencies() {
        let animation = TimelineAnimation::new("test".to_string(), AnimationConfig::default())
            .depends_on(vec!["first".to_string(), "second".to_string()]);
        
        assert!(!animation.can_start(&[]));
        assert!(!animation.can_start(&["first".to_string()]));
        assert!(animation.can_start(&["first".to_string(), "second".to_string()]));
    }

    #[test]
    fn test_parallel_groups() {
        let mut timeline = TimelineBuilder::new()
            .add_parallel("anim1".to_string(), AnimationConfig::default(), "group1".to_string())
            .add_parallel("anim2".to_string(), AnimationConfig::default(), "group1".to_string())
            .build();
        
        let group_animations = timeline.parallel_group_animations("group1");
        assert_eq!(group_animations.len(), 2);
        assert!(!timeline.is_parallel_group_completed("group1"));
    }
}