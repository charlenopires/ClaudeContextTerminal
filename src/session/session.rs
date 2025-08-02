//! Session management and persistence

use anyhow::Result;
use std::{path::Path, sync::Arc, collections::HashMap};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    llm::{Message, TokenUsage},
    session::database::{Database, SessionRow},
};

/// A conversation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub parent_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u32,
    pub token_usage: TokenUsage,
    pub total_cost: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new(title: String, parent_session_id: Option<String>) -> Self {
        let now = Utc::now();
        
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            parent_session_id,
            created_at: now,
            updated_at: now,
            message_count: 0,
            token_usage: TokenUsage::default(),
            total_cost: 0.0,
            metadata: HashMap::new(),
        }
    }
    
    /// Update token usage and cost
    pub fn update_usage(&mut self, usage: &TokenUsage, cost: f64) {
        self.token_usage.add(usage);
        self.total_cost += cost;
        self.updated_at = Utc::now();
    }
    
    /// Increment message count
    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
        self.updated_at = Utc::now();
    }
    
    /// Set metadata
    pub fn set_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }
    
    /// Get metadata
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

impl From<SessionRow> for Session {
    fn from(row: SessionRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            parent_session_id: row.parent_session_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
            message_count: row.message_count as u32,
            token_usage: TokenUsage {
                input_tokens: row.total_input_tokens as u32,
                output_tokens: row.total_output_tokens as u32,
                total_tokens: (row.total_input_tokens + row.total_output_tokens) as u32,
            },
            total_cost: row.total_cost,
            metadata: if let Some(metadata) = row.metadata {
                if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(metadata) {
                    map
                } else {
                    HashMap::new()
                }
            } else {
                HashMap::new()
            },
        }
    }
}

/// Session manager for handling session persistence and operations
pub struct SessionManager {
    db: Arc<Database>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub async fn new<P: AsRef<Path>>(data_dir: P) -> Result<Self> {
        let db_path = data_dir.as_ref().join("sessions.db");
        let db = Arc::new(Database::new(db_path).await?);
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        
        Ok(Self { db, sessions })
    }
    
    /// Create a new session
    pub async fn create_session(
        &self,
        title: String,
        parent_session_id: Option<String>,
    ) -> Result<Session> {
        let session = Session::new(title, parent_session_id);
        
        // Insert into database
        self.db.insert_session(
            &session.id,
            &session.title,
            session.parent_session_id.as_deref(),
            Some(&serde_json::to_value(&session.metadata)?),
        ).await?;
        
        // Cache in memory
        self.sessions.write().await.insert(session.id.clone(), session.clone());
        
        Ok(session)
    }
    
    /// Get a session by ID
    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        // Check cache first
        if let Some(session) = self.sessions.read().await.get(id) {
            return Ok(Some(session.clone()));
        }
        
        // Load from database
        if let Some(row) = self.db.get_session(id).await? {
            let session = Session::from(row);
            self.sessions.write().await.insert(id.to_string(), session.clone());
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }
    
    /// Update a session
    pub async fn update_session(&self, session: &Session) -> Result<()> {
        // Update database
        self.db.update_session(
            &session.id,
            Some(&session.title),
            Some(session.message_count as i32),
            Some(session.token_usage.input_tokens as i32),
            Some(session.token_usage.output_tokens as i32),
            Some(session.total_cost),
            Some(&serde_json::to_value(&session.metadata)?),
        ).await?;
        
        // Update cache
        self.sessions.write().await.insert(session.id.clone(), session.clone());
        
        Ok(())
    }
    
    /// List sessions
    pub async fn list_sessions(&self, limit: Option<u32>) -> Result<Vec<Session>> {
        let rows = self.db.list_sessions(limit.map(|l| l as i32)).await?;
        let sessions: Vec<Session> = rows.into_iter().map(Session::from).collect();
        
        // Update cache
        {
            let mut cache = self.sessions.write().await;
            for session in &sessions {
                cache.insert(session.id.clone(), session.clone());
            }
        }
        
        Ok(sessions)
    }
    
    /// Delete a session
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        // Delete from database
        self.db.delete_session(id).await?;
        
        // Remove from cache
        self.sessions.write().await.remove(id);
        
        Ok(())
    }
    
    /// Add a message to a session
    pub async fn add_message(&self, session_id: &str, message: &Message) -> Result<()> {
        // Insert message into database
        self.db.insert_message(message, session_id).await?;
        
        // Update session message count
        if let Some(mut session) = self.get_session(session_id).await? {
            session.increment_message_count();
            self.update_session(&session).await?;
        }
        
        Ok(())
    }
    
    /// Get messages for a session
    pub async fn get_messages(&self, session_id: &str, limit: Option<u32>) -> Result<Vec<Message>> {
        self.db.get_messages(session_id, limit.map(|l| l as i32)).await
    }
    
    /// Update session usage
    pub async fn update_session_usage(
        &self,
        session_id: &str,
        usage: &TokenUsage,
        cost: f64,
    ) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.update_usage(usage, cost);
            self.update_session(&session).await?;
        }
        
        Ok(())
    }
    
    /// Set session metadata
    pub async fn set_session_metadata(
        &self,
        session_id: &str,
        key: String,
        value: serde_json::Value,
    ) -> Result<()> {
        if let Some(mut session) = self.get_session(session_id).await? {
            session.set_metadata(key, value);
            self.update_session(&session).await?;
        }
        
        Ok(())
    }
    
    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &str) -> Result<Option<SessionStats>> {
        if let Some(session) = self.get_session(session_id).await? {
            let message_count = self.db.get_message_count(session_id).await? as u32;
            
            Ok(Some(SessionStats {
                session_id: session.id,
                message_count,
                token_usage: session.token_usage,
                total_cost: session.total_cost,
                created_at: session.created_at,
                updated_at: session.updated_at,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub message_count: u32,
    pub token_usage: TokenUsage,
    pub total_cost: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}