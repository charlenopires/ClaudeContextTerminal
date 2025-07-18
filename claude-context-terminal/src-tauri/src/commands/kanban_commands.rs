use crate::kanban::{Task, TaskStatus, TaskPriority, database};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub tags: Vec<String>,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
    pub prompt: Option<String>,
    pub claude_context: Option<String>,
}

#[tauri::command]
pub async fn get_tasks() -> Result<Vec<Task>, String> {
    let pool = database::get_pool();
    
    match sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
    {
        Ok(tasks) => Ok(tasks),
        Err(e) => Err(format!("Failed to fetch tasks: {}", e))
    }
}

#[tauri::command]
pub async fn create_task(request: CreateTaskRequest) -> Result<Task, String> {
    let pool = database::get_pool();
    let now = Utc::now();
    
    let task = Task {
        id: Uuid::new_v4().to_string(),
        title: request.title,
        description: request.description,
        status: TaskStatus::Todo,
        priority: request.priority,
        tags: serde_json::to_string(&request.tags).unwrap_or_else(|_| "[]".to_string()),
        prompt: request.prompt,
        claude_context: None,
        created_at: now,
        updated_at: now,
        completed_at: None,
        estimated_time: None,
        actual_time: None,
    };

    let result = sqlx::query(
        "INSERT INTO tasks (id, title, description, status, priority, tags, prompt, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&task.id)
    .bind(&task.title)
    .bind(&task.description)
    .bind(&task.status.to_string())
    .bind(&task.priority.to_string())
    .bind(&task.tags)
    .bind(&task.prompt)
    .bind(&task.created_at)
    .bind(&task.updated_at)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(task),
        Err(e) => Err(format!("Failed to create task: {}", e))
    }
}

#[tauri::command]
pub async fn update_task(request: UpdateTaskRequest) -> Result<Task, String> {
    let pool = database::get_pool();
    let now = Utc::now();

    // First, get the existing task
    let existing_task = match sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
        .bind(&request.id)
        .fetch_one(pool)
        .await
    {
        Ok(task) => task,
        Err(_) => return Err("Task not found".to_string())
    };

    let updated_task = Task {
        id: request.id.clone(),
        title: request.title.unwrap_or(existing_task.title),
        description: request.description.unwrap_or(existing_task.description),
        status: request.status.unwrap_or(existing_task.status),
        priority: request.priority.unwrap_or(existing_task.priority),
        tags: request.tags
            .map(|tags| serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()))
            .unwrap_or(existing_task.tags),
        prompt: request.prompt.unwrap_or(existing_task.prompt),
        claude_context: request.claude_context.or(existing_task.claude_context),
        created_at: existing_task.created_at,
        updated_at: now,
        completed_at: if request.status == Some(TaskStatus::Done) && existing_task.completed_at.is_none() {
            Some(now)
        } else if request.status.is_some() && request.status != Some(TaskStatus::Done) {
            None
        } else {
            existing_task.completed_at
        },
        estimated_time: existing_task.estimated_time,
        actual_time: existing_task.actual_time,
    };

    let result = sqlx::query(
        "UPDATE tasks SET title = ?, description = ?, status = ?, priority = ?, tags = ?, 
         prompt = ?, claude_context = ?, updated_at = ?, completed_at = ? WHERE id = ?"
    )
    .bind(&updated_task.title)
    .bind(&updated_task.description)
    .bind(&updated_task.status.to_string())
    .bind(&updated_task.priority.to_string())
    .bind(&updated_task.tags)
    .bind(&updated_task.prompt)
    .bind(&updated_task.claude_context)
    .bind(&updated_task.updated_at)
    .bind(&updated_task.completed_at)
    .bind(&updated_task.id)
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(updated_task),
        Err(e) => Err(format!("Failed to update task: {}", e))
    }
}

#[tauri::command]
pub async fn delete_task(id: String) -> Result<(), String> {
    let pool = database::get_pool();
    
    match sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(&id)
        .execute(pool)
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to delete task: {}", e))
    }
}

#[tauri::command]
pub async fn sync_with_markdown(file_path: String) -> Result<String, String> {
    // This will be implemented to sync with tasklist.md
    // For now, return a placeholder
    Ok("Sync functionality to be implemented".to_string())
}