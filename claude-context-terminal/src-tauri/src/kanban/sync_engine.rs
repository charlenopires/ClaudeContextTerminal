use super::{Task, TaskStatus, TaskPriority};
use anyhow::Result;
use std::path::Path;

pub struct SyncEngine;

impl SyncEngine {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn sync_tasks_to_markdown(&self, tasks: &[Task], file_path: &Path) -> Result<()> {
        let mut content = String::from("# Task List\n\n");
        
        // Group tasks by status
        let todo_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Todo).collect();
        let in_progress_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).collect();
        let done_tasks: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Done).collect();
        
        // To Do section
        content.push_str("## To Do\n");
        for task in todo_tasks {
            content.push_str(&self.format_task_for_markdown(task));
        }
        content.push('\n');
        
        // In Progress section
        content.push_str("## In Progress\n");
        for task in in_progress_tasks {
            content.push_str(&self.format_task_for_markdown(task));
        }
        content.push('\n');
        
        // Done section
        content.push_str("## Done\n");
        for task in done_tasks {
            content.push_str(&self.format_task_for_markdown(task));
        }
        
        std::fs::write(file_path, content)?;
        Ok(())
    }
    
    pub async fn sync_markdown_to_tasks(&self, file_path: &Path) -> Result<Vec<Task>> {
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = std::fs::read_to_string(file_path)?;
        let tasks = self.parse_markdown_tasks(&content)?;
        
        Ok(tasks)
    }
    
    fn format_task_for_markdown(&self, task: &Task) -> String {
        let status_char = match task.status {
            TaskStatus::Todo | TaskStatus::InProgress => "[ ]",
            TaskStatus::Done => "[x]",
        };
        
        let priority_tag = format!("[{}]", task.priority.to_string().to_uppercase());
        
        let tags: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();
        let tags_str = if !tags.is_empty() {
            format!(" {}", tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
        } else {
            String::new()
        };
        
        format!("- {} {} {}{}\n  - Descrição: {}\n  - Prompt: {}\n", 
            status_char, priority_tag, task.title, tags_str, task.description, task.prompt)
    }
    
    fn parse_markdown_tasks(&self, content: &str) -> Result<Vec<Task>> {
        // This is a simplified parser - in a real implementation, 
        // you'd want more robust markdown parsing
        let mut tasks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        let mut current_status = TaskStatus::Todo;
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Detect section headers
            if line.starts_with("## To Do") {
                current_status = TaskStatus::Todo;
            } else if line.starts_with("## In Progress") {
                current_status = TaskStatus::InProgress;
            } else if line.starts_with("## Done") {
                current_status = TaskStatus::Done;
            } else if line.starts_with("- [") {
                // Parse task line
                if let Some(task) = self.parse_task_line(line, current_status.clone()) {
                    tasks.push(task);
                }
            }
            
            i += 1;
        }
        
        Ok(tasks)
    }
    
    fn parse_task_line(&self, line: &str, status: TaskStatus) -> Option<Task> {
        // Very basic parsing - would need improvement for production use
        if let Some(task_part) = line.strip_prefix("- [ ] ").or_else(|| line.strip_prefix("- [x] ")) {
            let parts: Vec<&str> = task_part.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let priority_str = parts[0].trim_matches(['[', ']']);
                let priority = priority_str.parse().unwrap_or(TaskPriority::Medium);
                let title = parts[1].to_string();
                
                return Some(Task {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    description: String::new(),
                    status,
                    priority,
                    tags: "[]".to_string(),
                    prompt: String::new(),
                    claude_context: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    completed_at: None,
                    estimated_time: None,
                    actual_time: None,
                });
            }
        }
        
        None
    }
}