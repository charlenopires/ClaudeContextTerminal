use super::{PromptHook, HookTrigger, TriggerType};
use std::collections::HashMap;

pub struct HookManager {
    hooks: HashMap<String, PromptHook>,
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }
    
    pub fn add_hook(&mut self, hook: PromptHook) {
        self.hooks.insert(hook.id.clone(), hook);
    }
    
    pub fn remove_hook(&mut self, id: &str) -> Option<PromptHook> {
        self.hooks.remove(id)
    }
    
    pub fn get_hook(&self, id: &str) -> Option<&PromptHook> {
        self.hooks.get(id)
    }
    
    pub fn list_hooks(&self) -> Vec<&PromptHook> {
        self.hooks.values().collect()
    }
    
    pub fn find_matching_hooks(&self, context: &HookContext) -> Vec<&PromptHook> {
        self.hooks.values()
            .filter(|hook| self.matches_trigger(&hook.trigger, context))
            .collect()
    }
    
    fn matches_trigger(&self, trigger: &HookTrigger, context: &HookContext) -> bool {
        match &trigger.trigger_type {
            TriggerType::Keyword => {
                context.keywords.iter().any(|k| k.contains(&trigger.value))
            }
            TriggerType::FilePattern => {
                context.file_patterns.iter().any(|p| p.contains(&trigger.value))
            }
            TriggerType::TaskTag => {
                context.task_tags.iter().any(|t| t.contains(&trigger.value))
            }
        }
    }
}

#[derive(Debug)]
pub struct HookContext {
    pub keywords: Vec<String>,
    pub file_patterns: Vec<String>,
    pub task_tags: Vec<String>,
}

impl HookContext {
    pub fn new() -> Self {
        Self {
            keywords: Vec::new(),
            file_patterns: Vec::new(),
            task_tags: Vec::new(),
        }
    }
    
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
    
    pub fn with_file_patterns(mut self, patterns: Vec<String>) -> Self {
        self.file_patterns = patterns;
        self
    }
    
    pub fn with_task_tags(mut self, tags: Vec<String>) -> Self {
        self.task_tags = tags;
        self
    }
}