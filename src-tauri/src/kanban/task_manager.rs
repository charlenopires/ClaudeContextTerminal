use super::{Task, TaskStatus, TaskPriority};
use anyhow::Result;

pub struct TaskManager;

impl TaskManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_task(&self, task: &Task) -> Result<()> {
        if task.title.trim().is_empty() {
            return Err(anyhow::anyhow!("Task title cannot be empty"));
        }
        
        if task.title.len() > 200 {
            return Err(anyhow::anyhow!("Task title is too long"));
        }
        
        if task.description.len() > 2000 {
            return Err(anyhow::anyhow!("Task description is too long"));
        }
        
        Ok(())
    }
    
    pub fn format_task_for_markdown(&self, task: &Task) -> String {
        let status_char = match task.status {
            TaskStatus::Todo => "[ ]",
            TaskStatus::InProgress => "[ ]", // In progress is still shown as unchecked in markdown
            TaskStatus::Done => "[x]",
        };
        
        let priority_tag = format!("[{}]", task.priority.to_string().to_uppercase());
        let tags = if !task.tags.is_empty() {
            let tags: Vec<String> = serde_json::from_str(&task.tags).unwrap_or_default();
            if !tags.is_empty() {
                format!(" {}", tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        format!("- {} {} {}{}\n  - Descrição: {}\n  - Prompt: {}\n", 
            status_char, priority_tag, task.title, tags, task.description, task.prompt)
    }
}