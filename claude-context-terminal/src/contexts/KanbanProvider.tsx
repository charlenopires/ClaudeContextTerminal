import React, { createContext, useContext as useReactContext, useReducer, useCallback } from 'react';
import type { Task, CreateTaskRequest, UpdateTaskRequest } from '../types';
import { kanbanApi } from '../services/api';

interface KanbanState {
  tasks: Task[];
  isLoading: boolean;
  error: string | null;
  selectedTask: Task | null;
}

type KanbanAction =
  | { type: 'SET_TASKS'; payload: Task[] }
  | { type: 'ADD_TASK'; payload: Task }
  | { type: 'UPDATE_TASK'; payload: Task }
  | { type: 'DELETE_TASK'; payload: string }
  | { type: 'SET_SELECTED_TASK'; payload: Task | null }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null };

const initialState: KanbanState = {
  tasks: [],
  isLoading: false,
  error: null,
  selectedTask: null,
};

const kanbanReducer = (state: KanbanState, action: KanbanAction): KanbanState => {
  switch (action.type) {
    case 'SET_TASKS':
      return { ...state, tasks: action.payload };
    case 'ADD_TASK':
      return { ...state, tasks: [...state.tasks, action.payload] };
    case 'UPDATE_TASK':
      return {
        ...state,
        tasks: state.tasks.map(task => 
          task.id === action.payload.id ? action.payload : task
        ),
        selectedTask: state.selectedTask?.id === action.payload.id ? action.payload : state.selectedTask
      };
    case 'DELETE_TASK':
      return {
        ...state,
        tasks: state.tasks.filter(task => task.id !== action.payload),
        selectedTask: state.selectedTask?.id === action.payload ? null : state.selectedTask
      };
    case 'SET_SELECTED_TASK':
      return { ...state, selectedTask: action.payload };
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload };
    default:
      return state;
  }
};

interface KanbanContextType {
  state: KanbanState;
  loadTasks: () => Promise<void>;
  createTask: (request: CreateTaskRequest) => Promise<void>;
  updateTask: (request: UpdateTaskRequest) => Promise<void>;
  deleteTask: (id: string) => Promise<void>;
  moveTask: (taskId: string, newStatus: Task['status']) => Promise<void>;
  selectTask: (task: Task | null) => void;
}

const KanbanContext = createContext<KanbanContextType | undefined>(undefined);

export const KanbanProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(kanbanReducer, initialState);

  const loadTasks = useCallback(async () => {
    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      const tasks = await kanbanApi.getTasks();
      dispatch({ type: 'SET_TASKS', payload: tasks });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, []);

  const createTask = useCallback(async (request: CreateTaskRequest) => {
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      const task = await kanbanApi.createTask(request);
      dispatch({ type: 'ADD_TASK', payload: task });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    }
  }, []);

  const updateTask = useCallback(async (request: UpdateTaskRequest) => {
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      const task = await kanbanApi.updateTask(request);
      dispatch({ type: 'UPDATE_TASK', payload: task });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    }
  }, []);

  const deleteTask = useCallback(async (id: string) => {
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      await kanbanApi.deleteTask(id);
      dispatch({ type: 'DELETE_TASK', payload: id });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    }
  }, []);

  const moveTask = useCallback(async (taskId: string, newStatus: Task['status']) => {
    await updateTask({ id: taskId, status: newStatus });
  }, [updateTask]);

  const selectTask = useCallback((task: Task | null) => {
    dispatch({ type: 'SET_SELECTED_TASK', payload: task });
  }, []);

  const value: KanbanContextType = {
    state,
    loadTasks,
    createTask,
    updateTask,
    deleteTask,
    moveTask,
    selectTask,
  };

  return (
    <KanbanContext.Provider value={value}>
      {children}
    </KanbanContext.Provider>
  );
};

export const useKanban = (): KanbanContextType => {
  const context = useReactContext(KanbanContext);
  if (!context) {
    throw new Error('useKanban must be used within a KanbanProvider');
  }
  return context;
};