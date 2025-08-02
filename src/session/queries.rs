//! Query builder and type-safe database access patterns
//!
//! This module provides SQLC-like patterns for type-safe database operations
//! with compile-time query validation and strong typing.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row, ToSql};
use serde_json;
use std::collections::HashMap;

use crate::llm::{Message, MessageRole as Role, ContentBlock as MessageContent};
// use super::SessionRow; // Not yet defined

/// Type-safe query builder for sessions
pub struct SessionQueries<'conn> {
    conn: &'conn Connection,
}

impl<'conn> SessionQueries<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    /// Create a new session with compile-time validated parameters
    pub async fn create_session(&self, params: CreateSessionParams) -> Result<String> {
        let now = Utc::now().to_rfc3339();
        let metadata_str = params.metadata
            .map(|m| serde_json::to_string(&m))
            .transpose()?;

        self.conn.execute(
            "INSERT INTO sessions (
                id, title, parent_session_id, created_at, updated_at, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                params.id,
                params.title,
                params.parent_session_id,
                now,
                now,
                metadata_str
            ],
        )?;

        Ok(params.id)
    }

    /// Get session by ID with type safety
    pub async fn get_session_by_id(&self, id: &str) -> Result<Option<SessionDetails>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.title, s.parent_session_id, s.created_at, s.updated_at,
                    s.message_count, s.total_input_tokens, s.total_output_tokens,
                    s.total_cost, s.metadata,
                    COUNT(m.id) as actual_message_count
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             WHERE s.id = ?1
             GROUP BY s.id"
        )?;

        let mut session_iter = stmt.query_map([id], |row| {
            Ok(SessionDetails::from_row_with_stats(row)?)
        })?;

        match session_iter.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }

    /// List sessions with pagination and filtering
    pub async fn list_sessions(&self, params: ListSessionsParams) -> Result<Vec<SessionSummary>> {
        let mut query = String::from(
            "SELECT s.id, s.title, s.parent_session_id, s.created_at, s.updated_at,
                    s.message_count, s.total_input_tokens, s.total_output_tokens,
                    s.total_cost,
                    COUNT(m.id) as actual_message_count,
                    MAX(m.timestamp) as last_message_time
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id"
        );

        let mut conditions = Vec::new();
        let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(parent_id) = &params.parent_session_id {
            conditions.push("s.parent_session_id = ?");
            sql_params.push(Box::new(parent_id.clone()));
        }

        if let Some(since) = &params.created_since {
            conditions.push("s.created_at >= ?");
            sql_params.push(Box::new(since.to_rfc3339()));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" GROUP BY s.id ORDER BY s.updated_at DESC");

        if let Some(limit) = params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = params.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = self.conn.prepare(&query)?;
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        
        let session_iter = stmt.query_map(&param_refs[..], |row| {
            Ok(SessionSummary::from_row_with_stats(row)?)
        })?;

        let mut sessions = Vec::new();
        for session in session_iter {
            sessions.push(session?);
        }

        Ok(sessions)
    }

    /// Update session with builder pattern
    pub async fn update_session(&self, id: &str, updates: SessionUpdates) -> Result<bool> {
        let mut set_clauses = Vec::new();
        let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

        // Always update the updated_at timestamp
        set_clauses.push("updated_at = ?");
        sql_params.push(Box::new(Utc::now().to_rfc3339()));

        if let Some(title) = updates.title {
            set_clauses.push("title = ?");
            sql_params.push(Box::new(title));
        }

        if let Some(message_count) = updates.message_count {
            set_clauses.push("message_count = ?");
            sql_params.push(Box::new(message_count));
        }

        if let Some(input_tokens) = updates.total_input_tokens {
            set_clauses.push("total_input_tokens = ?");
            sql_params.push(Box::new(input_tokens));
        }

        if let Some(output_tokens) = updates.total_output_tokens {
            set_clauses.push("total_output_tokens = ?");
            sql_params.push(Box::new(output_tokens));
        }

        if let Some(cost) = updates.total_cost {
            set_clauses.push("total_cost = ?");
            sql_params.push(Box::new(cost));
        }

        if let Some(metadata) = updates.metadata {
            set_clauses.push("metadata = ?");
            let metadata_str = serde_json::to_string(&metadata)?;
            sql_params.push(Box::new(metadata_str));
        }

        // Add the ID parameter last
        sql_params.push(Box::new(id.to_string()));

        let query = format!(
            "UPDATE sessions SET {} WHERE id = ?",
            set_clauses.join(", ")
        );

        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let rows_affected = self.conn.execute(&query, &param_refs[..])?;

        Ok(rows_affected > 0)
    }

    /// Delete session and cascade to messages
    pub async fn delete_session(&self, id: &str) -> Result<bool> {
        let rows_affected = self.conn.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
        Ok(rows_affected > 0)
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, id: &str) -> Result<Option<SessionStats>> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(m.id) as message_count,
                MIN(m.timestamp) as first_message_time,
                MAX(m.timestamp) as last_message_time,
                s.total_input_tokens,
                s.total_output_tokens,
                s.total_cost
             FROM sessions s
             LEFT JOIN messages m ON s.id = m.session_id
             WHERE s.id = ?1
             GROUP BY s.id"
        )?;

        let mut stats_iter = stmt.query_map([id], |row| {
            Ok(SessionStats::from_row(row)?)
        })?;

        match stats_iter.next() {
            Some(result) => Ok(Some(result?)),
            None => Ok(None),
        }
    }
}

/// Type-safe query builder for messages
pub struct MessageQueries<'conn> {
    conn: &'conn Connection,
}

impl<'conn> MessageQueries<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }

    /// Insert a message with type safety
    pub async fn create_message(&self, params: CreateMessageParams) -> Result<String> {
        let content_str = serde_json::to_string(&params.content)?;
        let metadata_str = if params.metadata.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&params.metadata)?)
        };

        self.conn.execute(
            "INSERT INTO messages (id, session_id, role, content, timestamp, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                params.id,
                params.session_id,
                serde_json::to_string(&params.role)?,
                content_str,
                params.timestamp.to_rfc3339(),
                metadata_str
            ],
        )?;

        Ok(params.id)
    }

    /// Get messages with filtering and pagination
    pub async fn get_messages(&self, params: GetMessagesParams) -> Result<Vec<Message>> {
        let mut query = String::from(
            "SELECT id, role, content, timestamp, metadata
             FROM messages
             WHERE session_id = ?"
        );

        let mut sql_params: Vec<Box<dyn ToSql>> = vec![Box::new(params.session_id.clone())];

        if let Some(role) = &params.role_filter {
            query.push_str(" AND role = ?");
            sql_params.push(Box::new(serde_json::to_string(role)?));
        }

        if let Some(since) = &params.since {
            query.push_str(" AND timestamp >= ?");
            sql_params.push(Box::new(since.to_rfc3339()));
        }

        if let Some(until) = &params.until {
            query.push_str(" AND timestamp <= ?");
            sql_params.push(Box::new(until.to_rfc3339()));
        }

        query.push_str(" ORDER BY timestamp ASC");

        if let Some(limit) = params.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = params.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        let mut stmt = self.conn.prepare(&query)?;
        let param_refs: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();

        let message_iter = stmt.query_map(&param_refs[..], |row| {
            Ok(Message::from_row(row)?)
        })?;

        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }

        Ok(messages)
    }

    /// Get message statistics for a session
    pub async fn get_message_stats(&self, session_id: &str) -> Result<MessageStats> {
        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(*) as total_count,
                COUNT(CASE WHEN role = ? THEN 1 END) as user_count,
                COUNT(CASE WHEN role = ? THEN 1 END) as assistant_count,
                COUNT(CASE WHEN role = ? THEN 1 END) as system_count,
                MIN(timestamp) as first_message,
                MAX(timestamp) as last_message
             FROM messages 
             WHERE session_id = ?"
        )?;

        let user_role = serde_json::to_string(&Role::User)?;
        let assistant_role = serde_json::to_string(&Role::Assistant)?;
        let system_role = serde_json::to_string(&Role::System)?;

        let stats = stmt.query_row(
            params![user_role, assistant_role, system_role, session_id],
            |row| Ok(MessageStats::from_row(row)?)
        )?;

        Ok(stats)
    }

    /// Delete all messages for a session
    pub async fn delete_session_messages(&self, session_id: &str) -> Result<u32> {
        let rows_affected = self.conn.execute(
            "DELETE FROM messages WHERE session_id = ?1",
            [session_id]
        )?;

        Ok(rows_affected as u32)
    }
}

// Parameter types for type-safe queries
#[derive(Debug)]
pub struct CreateSessionParams {
    pub id: String,
    pub title: String,
    pub parent_session_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Default)]
pub struct ListSessionsParams {
    pub parent_session_id: Option<String>,
    pub created_since: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Default)]
pub struct SessionUpdates {
    pub title: Option<String>,
    pub message_count: Option<i32>,
    pub total_input_tokens: Option<i32>,
    pub total_output_tokens: Option<i32>,
    pub total_cost: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct CreateMessageParams {
    pub id: String,
    pub session_id: String,
    pub role: Role,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct GetMessagesParams {
    pub session_id: String,
    pub role_filter: Option<Role>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// Enhanced result types
#[derive(Debug, Clone)]
pub struct SessionDetails {
    pub id: String,
    pub title: String,
    pub parent_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: i32,
    pub actual_message_count: i32,
    pub total_input_tokens: i32,
    pub total_output_tokens: i32,
    pub total_cost: f64,
    pub metadata: Option<serde_json::Value>,
}

impl SessionDetails {
    fn from_row_with_stats(row: &Row) -> rusqlite::Result<Self> {
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;
        let metadata_str: Option<String> = row.get(9)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(3, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(4, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let metadata = if let Some(metadata_str) = metadata_str {
            Some(serde_json::from_str(&metadata_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(9, "metadata".to_string(), rusqlite::types::Type::Text))?)
        } else {
            None
        };

        Ok(SessionDetails {
            id: row.get(0)?,
            title: row.get(1)?,
            parent_session_id: row.get(2)?,
            created_at,
            updated_at,
            message_count: row.get(5)?,
            total_input_tokens: row.get(6)?,
            total_output_tokens: row.get(7)?,
            total_cost: row.get(8)?,
            metadata,
            actual_message_count: row.get(10)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub parent_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: i32,
    pub actual_message_count: i32,
    pub total_input_tokens: i32,
    pub total_output_tokens: i32,
    pub total_cost: f64,
    pub last_message_time: Option<DateTime<Utc>>,
}

impl SessionSummary {
    fn from_row_with_stats(row: &Row) -> rusqlite::Result<Self> {
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;
        let last_message_str: Option<String> = row.get(10)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(3, "created_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(4, "updated_at".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);

        let last_message_time = if let Some(time_str) = last_message_str {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(10, "last_message_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc))
        } else {
            None
        };

        Ok(SessionSummary {
            id: row.get(0)?,
            title: row.get(1)?,
            parent_session_id: row.get(2)?,
            created_at,
            updated_at,
            message_count: row.get(5)?,
            total_input_tokens: row.get(6)?,
            total_output_tokens: row.get(7)?,
            total_cost: row.get(8)?,
            actual_message_count: row.get(9)?,
            last_message_time,
        })
    }
}

#[derive(Debug)]
pub struct SessionStats {
    pub message_count: i32,
    pub first_message_time: Option<DateTime<Utc>>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub total_input_tokens: i32,
    pub total_output_tokens: i32,
    pub total_cost: f64,
}

impl SessionStats {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let first_str: Option<String> = row.get(1)?;
        let last_str: Option<String> = row.get(2)?;

        let first_message_time = if let Some(time_str) = first_str {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(1, "first_message_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc))
        } else {
            None
        };

        let last_message_time = if let Some(time_str) = last_str {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(2, "last_message_time".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc))
        } else {
            None
        };

        Ok(SessionStats {
            message_count: row.get(0)?,
            first_message_time,
            last_message_time,
            total_input_tokens: row.get(3)?,
            total_output_tokens: row.get(4)?,
            total_cost: row.get(5)?,
        })
    }
}

#[derive(Debug)]
pub struct MessageStats {
    pub total_count: i32,
    pub user_count: i32,
    pub assistant_count: i32,
    pub system_count: i32,
    pub first_message: Option<DateTime<Utc>>,
    pub last_message: Option<DateTime<Utc>>,
}

impl MessageStats {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let first_str: Option<String> = row.get(4)?;
        let last_str: Option<String> = row.get(5)?;

        let first_message = if let Some(time_str) = first_str {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "first_message".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc))
        } else {
            None
        };

        let last_message = if let Some(time_str) = last_str {
            Some(DateTime::parse_from_rfc3339(&time_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(5, "last_message".to_string(), rusqlite::types::Type::Text))?
                .with_timezone(&Utc))
        } else {
            None
        };

        Ok(MessageStats {
            total_count: row.get(0)?,
            user_count: row.get(1)?,
            assistant_count: row.get(2)?,
            system_count: row.get(3)?,
            first_message,
            last_message,
        })
    }
}

// Extension trait for Message to work with database rows
impl Message {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let id: String = row.get(0)?;
        let role_str: String = row.get(1)?;
        let content_str: String = row.get(2)?;
        let timestamp_str: String = row.get(3)?;
        let metadata_str: Option<String> = row.get(4)?;

        let role = serde_json::from_str(&role_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(1, "role".to_string(), rusqlite::types::Type::Text))?;
        let content = serde_json::from_str(&content_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(2, "content".to_string(), rusqlite::types::Type::Text))?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map_err(|_| rusqlite::Error::InvalidColumnType(3, "timestamp".to_string(), rusqlite::types::Type::Text))?
            .with_timezone(&Utc);
        let metadata = if let Some(metadata_str) = metadata_str {
            serde_json::from_str(&metadata_str)
                .map_err(|_| rusqlite::Error::InvalidColumnType(4, "metadata".to_string(), rusqlite::types::Type::Text))?
        } else {
            HashMap::new()
        };

        Ok(Message {
            id,
            role,
            content,
            timestamp,
            metadata,
        })
    }
}