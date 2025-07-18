import { invoke } from '@tauri-apps/api/core';
import type { 
  Task, 
  CreateTaskRequest, 
  UpdateTaskRequest, 
  ContextFile, 
  ClaudeMdContent,
  ClaudeSession,
  MCPServer,
  PromptHook
} from '../types';

// Context Management
export const contextApi = {
  loadContext: (filePath: string): Promise<string> => 
    invoke('load_context', { filePath }),
  
  saveContext: (filePath: string, content: string): Promise<void> => 
    invoke('save_context', { filePath, content }),
  
  listMdFiles: (directory: string): Promise<ContextFile[]> => 
    invoke('list_md_files', { directory }),
  
  generateClaudeMd: (directory: string): Promise<ClaudeMdContent> => 
    invoke('generate_claude_md', { directory }),
};

// Kanban Management
export const kanbanApi = {
  getTasks: (): Promise<Task[]> => 
    invoke('get_tasks'),
  
  createTask: (request: CreateTaskRequest): Promise<Task> => 
    invoke('create_task', { request }),
  
  updateTask: (request: UpdateTaskRequest): Promise<Task> => 
    invoke('update_task', { request }),
  
  deleteTask: (id: string): Promise<void> => 
    invoke('delete_task', { id }),
  
  syncWithMarkdown: (filePath: string): Promise<string> => 
    invoke('sync_with_markdown', { filePath }),
};

// Claude CLI Management
export const claudeApi = {
  startSession: (directory: string): Promise<ClaudeSession> => 
    invoke('start_claude_session', { directory }),
  
  executeTask: (request: {
    taskId: string;
    taskTitle: string;
    taskDescription: string;
    taskPrompt: string;
    contextFiles: string[];
    mcpServers: string[];
  }): Promise<string> => 
    invoke('execute_task', { request }),
  
  getSessionStatus: (sessionId: string): Promise<ClaudeSession> => 
    invoke('get_session_status', { sessionId }),
};

// MCP Management
export const mcpApi = {
  listServers: (): Promise<MCPServer[]> => 
    invoke('list_servers'),
  
  toggleServer: (serverName: string, enabled: boolean): Promise<void> => 
    invoke('toggle_server', { serverName, enabled }),
  
  createHook: (request: {
    name: string;
    triggerType: 'keyword' | 'file_pattern' | 'task_tag';
    triggerValue: string;
    template: string;
    mcpServers: string[];
  }): Promise<PromptHook> => 
    invoke('create_hook', { request }),
};