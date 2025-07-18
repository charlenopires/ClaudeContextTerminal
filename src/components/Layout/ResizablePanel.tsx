import React, { useRef, useCallback, useEffect } from 'react';

interface ResizablePanelProps {
  width: number; // percentage
  onResize: (newWidth: number) => void;
  className?: string;
  children: React.ReactNode;
}

export const ResizablePanel: React.FC<ResizablePanelProps> = ({
  width,
  onResize,
  className = '',
  children,
}) => {
  const isDragging = useRef(false);
  const panelRef = useRef<HTMLDivElement>(null);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }, []);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging.current || !panelRef.current) return;
    
    const container = panelRef.current.parentElement;
    if (!container) return;
    
    const containerRect = container.getBoundingClientRect();
    const newWidth = e.clientX - containerRect.left;
    onResize(newWidth);
  }, [onResize]);

  const handleMouseUp = useCallback(() => {
    isDragging.current = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }, []);

  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [handleMouseMove, handleMouseUp]);

  return (
    <div
      ref={panelRef}
      className={`relative ${className}`}
      style={{ width: `${width}%` }}
    >
      {children}
      
      {/* Resize Handle */}
      <div
        className="absolute top-0 right-0 w-1 h-full cursor-col-resize bg-gray-600 hover:bg-gray-500 transition-colors group"
        onMouseDown={handleMouseDown}
      >
        <div className="absolute top-1/2 right-0 w-1 h-8 bg-gray-400 transform -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity" />
      </div>
    </div>
  );
};