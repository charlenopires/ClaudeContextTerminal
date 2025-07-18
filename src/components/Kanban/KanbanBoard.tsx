import React, { useEffect } from 'react';
import { useKanban } from '../../contexts/KanbanProvider';
import { TaskCard } from './TaskCard';
import { Plus } from 'lucide-react';
import type { Task } from '../../types';

export const KanbanBoard: React.FC = () => {
  const { state, loadTasks, moveTask } = useKanban();
  const { tasks, isLoading, error } = state;

  useEffect(() => {
    loadTasks();
  }, [loadTasks]);

  const tasksByStatus = {
    todo: tasks.filter(task => task.status === 'todo'),
    in_progress: tasks.filter(task => task.status === 'in_progress'),
    done: tasks.filter(task => task.status === 'done'),
  };

  const handleTaskMove = async (taskId: string, newStatus: Task['status']) => {
    await moveTask(taskId, newStatus);
  };

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-gray-400">Loading tasks...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="h-full p-4 overflow-auto">
      <div className="flex gap-4 h-full">
        {/* To Do Column */}
        <div className="flex-1 min-w-80">
          <div className="kanban-column">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-white">To Do</h3>
              <button className="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded">
                <Plus className="w-4 h-4" />
              </button>
            </div>
            <div className="space-y-3">
              {tasksByStatus.todo.map(task => (
                <TaskCard
                  key={task.id}
                  task={task}
                  onMove={handleTaskMove}
                />
              ))}
            </div>
          </div>
        </div>

        {/* In Progress Column */}
        <div className="flex-1 min-w-80">
          <div className="kanban-column">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-white">In Progress</h3>
              <span className="text-sm text-gray-400">
                {tasksByStatus.in_progress.length}
              </span>
            </div>
            <div className="space-y-3">
              {tasksByStatus.in_progress.map(task => (
                <TaskCard
                  key={task.id}
                  task={task}
                  onMove={handleTaskMove}
                />
              ))}
            </div>
          </div>
        </div>

        {/* Done Column */}
        <div className="flex-1 min-w-80">
          <div className="kanban-column">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-white">Done</h3>
              <span className="text-sm text-gray-400">
                {tasksByStatus.done.length}
              </span>
            </div>
            <div className="space-y-3">
              {tasksByStatus.done.map(task => (
                <TaskCard
                  key={task.id}
                  task={task}
                  onMove={handleTaskMove}
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};