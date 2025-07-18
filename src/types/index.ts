export interface Task {
  id: string;
  title: string;
  description: string;
  status: 'todo' | 'in_progress' | 'done';
  priority: 'low' | 'medium' | 'high';
  tags: string[];
  prompt: string;
  claudeContext?: string;
  createdAt: string;
  updatedAt: string;
  completedAt?: string;
  estimatedTime?: number;
  actualTime?: number;
}

export interface ContextFile {
  path: string;
  name: string;
  content: string;
  lastModified: string;
}

export interface ClaudeMdContent {
  content: string;
  includedFiles: string[];
}

export interface ClaudeSession {
  id: string;
  status: 'active' | 'idle' | 'stopped' | 'error';
  currentDirectory: string;
  lastActivity: string;
  totalCommands: number;
}

export interface MCPServer {
  name: string;
  description: string;
  command: string;
  triggers: string[];
  enabled: boolean;
}

export interface PromptHook {
  id: string;
  name: string;
  trigger: {
    type: 'keyword' | 'file_pattern' | 'task_tag';
    value: string;
  };
  template: string;
  mcpServers: string[];
}

export interface CreateTaskRequest {
  title: string;
  description: string;
  priority: Task['priority'];
  tags: string[];
  prompt: string;
}

export interface UpdateTaskRequest {
  id: string;
  title?: string;
  description?: string;
  status?: Task['status'];
  priority?: Task['priority'];
  tags?: string[];
  prompt?: string;
  claudeContext?: string;
}