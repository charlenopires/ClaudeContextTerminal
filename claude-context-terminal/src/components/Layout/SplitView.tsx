import React, { useState, useRef, useCallback } from 'react';
import { ResizablePanel } from './ResizablePanel';
import { ContextManager } from '../ContextManager/ContextManager';
import { KanbanBoard } from '../Kanban/KanbanBoard';
import { ClaudeTerminal } from '../Terminal/ClaudeTerminal';
import { Toolbar } from './Toolbar';

export const SplitView: React.FC = () => {
  const [leftPanelWidth, setLeftPanelWidth] = useState(33); // 33% of screen width
  const [activeLeftTab, setActiveLeftTab] = useState<'context' | 'kanban'>('context');
  const containerRef = useRef<HTMLDivElement>(null);
  
  const handleResize = useCallback((newWidth: number) => {
    if (containerRef.current) {
      const containerWidth = containerRef.current.offsetWidth;
      const percentage = (newWidth / containerWidth) * 100;
      setLeftPanelWidth(Math.min(Math.max(percentage, 20), 60)); // Limit between 20% and 60%
    }
  }, []);

  return (
    <div className="h-screen flex flex-col bg-gray-900">
      <Toolbar />
      
      <div 
        ref={containerRef}
        className="flex-1 flex overflow-hidden"
      >
        {/* Left Panel */}
        <ResizablePanel
          width={leftPanelWidth}
          onResize={handleResize}
          className="bg-gray-850 border-r border-gray-700"
        >
          <div className="h-full flex flex-col">
            {/* Tab Bar */}
            <div className="flex border-b border-gray-700">
              <button
                onClick={() => setActiveLeftTab('context')}
                className={`px-4 py-2 text-sm font-medium transition-colors ${
                  activeLeftTab === 'context'
                    ? 'bg-gray-700 text-white border-b-2 border-blue-500'
                    : 'text-gray-400 hover:text-white hover:bg-gray-800'
                }`}
              >
                Context
              </button>
              <button
                onClick={() => setActiveLeftTab('kanban')}
                className={`px-4 py-2 text-sm font-medium transition-colors ${
                  activeLeftTab === 'kanban'
                    ? 'bg-gray-700 text-white border-b-2 border-blue-500'
                    : 'text-gray-400 hover:text-white hover:bg-gray-800'
                }`}
              >
                Kanban
              </button>
            </div>
            
            {/* Tab Content */}
            <div className="flex-1 overflow-hidden">
              {activeLeftTab === 'context' && <ContextManager />}
              {activeLeftTab === 'kanban' && <KanbanBoard />}
            </div>
          </div>
        </ResizablePanel>
        
        {/* Right Panel - Terminal */}
        <div className="flex-1 bg-terminal-bg">
          <ClaudeTerminal />
        </div>
      </div>
    </div>
  );
};