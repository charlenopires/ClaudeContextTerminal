use super::{ClaudeSession, SessionStatus};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, ClaudeSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn create_session(&self, directory: String) -> Result<ClaudeSession> {
        let session = ClaudeSession {
            id: uuid::Uuid::new_v4().to_string(),
            status: SessionStatus::Active,
            current_directory: directory,
            last_activity: chrono::Utc::now(),
            total_commands: 0,
        };
        
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.insert(session.id.clone(), session.clone());
        }
        
        Ok(session)
    }
    
    pub fn get_session(&self, session_id: &str) -> Option<ClaudeSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }
    
    pub fn update_session_activity(&self, session_id: &str) -> Result<()> {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.last_activity = chrono::Utc::now();
                session.total_commands += 1;
            }
        }
        Ok(())
    }
    
    pub fn set_session_status(&self, session_id: &str, status: SessionStatus) -> Result<()> {
        if let Ok(mut sessions) = self.sessions.lock() {
            if let Some(session) = sessions.get_mut(session_id) {
                session.status = status;
                session.last_activity = chrono::Utc::now();
            }
        }
        Ok(())
    }
}