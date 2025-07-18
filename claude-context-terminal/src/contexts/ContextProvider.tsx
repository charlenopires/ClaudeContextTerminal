import React, { createContext, useContext as useReactContext, useReducer, useCallback } from 'react';
import type { ContextFile, ClaudeMdContent } from '../types';
import { contextApi } from '../services/api';

interface ContextState {
  currentDirectory: string;
  contextFiles: ContextFile[];
  claudeMdContent: ClaudeMdContent | null;
  isLoading: boolean;
  error: string | null;
}

type ContextAction =
  | { type: 'SET_DIRECTORY'; payload: string }
  | { type: 'SET_CONTEXT_FILES'; payload: ContextFile[] }
  | { type: 'SET_CLAUDE_MD'; payload: ClaudeMdContent }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null };

const initialState: ContextState = {
  currentDirectory: '',
  contextFiles: [],
  claudeMdContent: null,
  isLoading: false,
  error: null,
};

const contextReducer = (state: ContextState, action: ContextAction): ContextState => {
  switch (action.type) {
    case 'SET_DIRECTORY':
      return { ...state, currentDirectory: action.payload };
    case 'SET_CONTEXT_FILES':
      return { ...state, contextFiles: action.payload };
    case 'SET_CLAUDE_MD':
      return { ...state, claudeMdContent: action.payload };
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload };
    default:
      return state;
  }
};

interface ContextContextType {
  state: ContextState;
  setDirectory: (directory: string) => void;
  loadContextFiles: () => Promise<void>;
  generateClaudeMd: () => Promise<void>;
  saveContext: (filePath: string, content: string) => Promise<void>;
}

const ContextContext = createContext<ContextContextType | undefined>(undefined);

export const ContextProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(contextReducer, initialState);

  const setDirectory = useCallback((directory: string) => {
    dispatch({ type: 'SET_DIRECTORY', payload: directory });
  }, []);

  const loadContextFiles = useCallback(async () => {
    if (!state.currentDirectory) return;
    
    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      const files = await contextApi.listMdFiles(state.currentDirectory);
      dispatch({ type: 'SET_CONTEXT_FILES', payload: files });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, [state.currentDirectory]);

  const generateClaudeMd = useCallback(async () => {
    if (!state.currentDirectory) return;
    
    dispatch({ type: 'SET_LOADING', payload: true });
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      const claudeMd = await contextApi.generateClaudeMd(state.currentDirectory);
      dispatch({ type: 'SET_CLAUDE_MD', payload: claudeMd });
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    } finally {
      dispatch({ type: 'SET_LOADING', payload: false });
    }
  }, [state.currentDirectory]);

  const saveContext = useCallback(async (filePath: string, content: string) => {
    dispatch({ type: 'SET_ERROR', payload: null });
    
    try {
      await contextApi.saveContext(filePath, content);
    } catch (error) {
      dispatch({ type: 'SET_ERROR', payload: error as string });
    }
  }, []);

  const value: ContextContextType = {
    state,
    setDirectory,
    loadContextFiles,
    generateClaudeMd,
    saveContext,
  };

  return (
    <ContextContext.Provider value={value}>
      {children}
    </ContextContext.Provider>
  );
};

export const useContext = (): ContextContextType => {
  const context = useReactContext(ContextContext);
  if (!context) {
    throw new Error('useContext must be used within a ContextProvider');
  }
  return context;
};