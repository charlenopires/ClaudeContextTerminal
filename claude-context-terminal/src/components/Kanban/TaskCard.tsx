import React from 'react';
import { Play, MoreVertical, Clock, Tag } from 'lucide-react';
import type { Task } from '../../types';

interface TaskCardProps {
  task: Task;
  onMove: (taskId: string, newStatus: Task['status']) => void;
}

export const TaskCard: React.FC<TaskCardProps> = ({ task, onMove }) => {
  const getPriorityColor = (priority: Task['priority']) => {
    switch (priority) {
      case 'high': return 'border-red-500 bg-red-500/10';
      case 'medium': return 'border-yellow-500 bg-yellow-500/10';
      case 'low': return 'border-green-500 bg-green-500/10';
      default: return 'border-gray-500 bg-gray-500/10';
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const handleExecuteTask = () => {
    // Move to in_progress if not already there
    if (task.status !== 'in_progress') {
      onMove(task.id, 'in_progress');
    }
    // TODO: Execute Claude CLI with this task
  };

  return (
    <div className={`kanban-card border-l-4 ${getPriorityColor(task.priority)}`}>
      <div className="flex items-start justify-between mb-2">
        <h4 className="font-medium text-white text-sm line-clamp-2">
          {task.title}
        </h4>
        <button className="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded">
          <MoreVertical className="w-3 h-3" />
        </button>
      </div>
      
      {task.description && (
        <p className="text-gray-300 text-xs mb-3 line-clamp-3">
          {task.description}
        </p>
      )}
      
      {task.tags && task.tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-3">
          {task.tags.slice(0, 3).map((tag, index) => (
            <span
              key={index}
              className="inline-flex items-center gap-1 px-2 py-1 bg-gray-700 text-gray-200 text-xs rounded-full"
            >
              <Tag className="w-2 h-2" />
              {tag}
            </span>
          ))}
          {task.tags.length > 3 && (
            <span className="text-gray-400 text-xs">+{task.tags.length - 3}</span>
          )}
        </div>
      )}
      
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 text-gray-400 text-xs">
          <Clock className="w-3 h-3" />
          <span>{formatDate(task.createdAt)}</span>
        </div>
        
        {task.status !== 'done' && (
          <button
            onClick={handleExecuteTask}
            className="inline-flex items-center gap-1 px-2 py-1 bg-blue-600 hover:bg-blue-700 text-white text-xs rounded transition-colors"
          >
            <Play className="w-3 h-3" />
            Execute
          </button>
        )}
      </div>
      
      {task.prompt && (
        <div className="mt-2 pt-2 border-t border-gray-700">
          <p className="text-gray-400 text-xs">
            Prompt: {task.prompt.substring(0, 50)}
            {task.prompt.length > 50 && '...'}
          </p>
        </div>
      )}
    </div>
  );
};