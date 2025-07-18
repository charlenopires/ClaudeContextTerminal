use crate::claude::{ClaudeSession, SessionStatus, TaskExecutionRequest};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

static mut SESSIONS: Option<Arc<Mutex<HashMap<String, ClaudeSession>>>> = None;

fn get_sessions() -> &'static Arc<Mutex<HashMap<String, ClaudeSession>>> {
    unsafe {
        if SESSIONS.is_none() {
            SESSIONS = Some(Arc::new(Mutex::new(HashMap::new())));
        }
        SESSIONS.as_ref().unwrap()
    }
}

#[tauri::command]
pub async fn start_claude_session(directory: String) -> Result<ClaudeSession, String> {
    let session_id = Uuid::new_v4().to_string();
    let session = ClaudeSession {
        id: session_id.clone(),
        status: SessionStatus::Active,
        current_directory: directory,
        last_activity: chrono::Utc::now(),
        total_commands: 0,
    };

    let sessions = get_sessions();
    if let Ok(mut sessions_map) = sessions.lock() {
        sessions_map.insert(session_id, session.clone());
    }

    Ok(session)
}

#[tauri::command]
pub async fn execute_task(request: TaskExecutionRequest) -> Result<String, String> {
    // Build the contextual prompt
    let prompt = build_contextual_prompt(&request).await?;
    
    // For now, return the prompt that would be sent to Claude
    // In the full implementation, this would execute Claude CLI
    Ok(format!("Would execute Claude with prompt:\n\n{}", prompt))
}

#[tauri::command]
pub async fn get_session_status(session_id: String) -> Result<ClaudeSession, String> {
    let sessions = get_sessions();
    if let Ok(sessions_map) = sessions.lock() {
        if let Some(session) = sessions_map.get(&session_id) {
            Ok(session.clone())
        } else {
            Err("Session not found".to_string())
        }
    } else {
        Err("Failed to access sessions".to_string())
    }
}

async fn build_contextual_prompt(request: &TaskExecutionRequest) -> Result<String, String> {
    let mut prompt = String::new();
    
    prompt.push_str("[CONTEXTO DO PROJETO]\n");
    for file_path in &request.context_files {
        if let Ok(content) = std::fs::read_to_string(file_path) {
            prompt.push_str(&format!("## {}\n{}\n\n", file_path, content));
        }
    }
    
    prompt.push_str("[TAREFA ATUAL]\n");
    prompt.push_str(&format!("Título: {}\n", request.task_title));
    prompt.push_str(&format!("Descrição: {}\n", request.task_description));
    prompt.push_str("\n[INSTRUÇÕES]\n");
    prompt.push_str(&request.task_prompt);
    
    if !request.mcp_servers.is_empty() {
        prompt.push_str("\n\n[SERVIDORES MCP DISPONÍVEIS]\n");
        for server in &request.mcp_servers {
            prompt.push_str(&format!("- @{}\n", server));
        }
    }
    
    Ok(prompt)
}