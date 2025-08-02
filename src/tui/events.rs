use crossterm::event::{KeyEvent, MouseEvent, Event as CrosstermEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::timeout;
use anyhow::Result;

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// Keyboard input event
    Key(KeyEvent),
    
    /// Mouse input event  
    Mouse(MouseEvent),
    
    /// Terminal resize event
    Resize(u16, u16),
    
    /// Periodic tick event
    Tick,
    
    /// Page navigation event
    PageChange(String),
    
    /// Status message event
    StatusMessage(String),
    
    /// Clear status message event
    ClearStatus,
    
    /// Custom application events
    Custom(String, serde_json::Value),
}

/// Event handler for managing input events
pub struct EventHandler {
    /// Event receiver channel
    receiver: mpsc::UnboundedReceiver<Event>,
    
    /// Event sender channel
    sender: mpsc::UnboundedSender<Event>,
    
    /// Tick interval for periodic events
    tick_interval: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let tick_interval = Duration::from_millis(100); // 10 FPS
        
        Self {
            receiver,
            sender,
            tick_interval,
        }
    }
    
    /// Get the next event
    pub async fn next(&mut self) -> Option<Event> {
        // Try to get crossterm events with timeout
        if let Ok(Ok(crossterm_event)) = timeout(
            Duration::from_millis(50),
            tokio::task::spawn_blocking(|| crossterm::event::read())
        ).await {
            if let Ok(event) = crossterm_event {
                return Some(self.convert_crossterm_event(event));
            }
        }
        
        // Check for internal events
        if let Ok(event) = self.receiver.try_recv() {
            return Some(event);
        }
        
        // Return tick event if no other events
        Some(Event::Tick)
    }
    
    /// Convert crossterm events to application events
    fn convert_crossterm_event(&self, event: CrosstermEvent) -> Event {
        match event {
            CrosstermEvent::Key(key_event) => Event::Key(key_event),
            CrosstermEvent::Mouse(mouse_event) => Event::Mouse(mouse_event),
            CrosstermEvent::Resize(width, height) => Event::Resize(width, height),
            CrosstermEvent::FocusGained => Event::Custom("focus_gained".to_string(), serde_json::Value::Null),
            CrosstermEvent::FocusLost => Event::Custom("focus_lost".to_string(), serde_json::Value::Null),
            CrosstermEvent::Paste(text) => Event::Custom("paste".to_string(), serde_json::Value::String(text)),
        }
    }
    
    /// Send an internal event
    pub fn send(&self, event: Event) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }
    
    /// Get a clone of the sender
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}