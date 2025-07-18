import React from 'react';
import { Folder, Terminal, Server, Settings } from 'lucide-react';

export const Toolbar: React.FC = () => {
  return (
    <div className="h-12 bg-gray-800 border-b border-gray-700 flex items-center px-4 gap-4">
      {/* App Title */}
      <div className="flex items-center gap-2">
        <Terminal className="w-5 h-5 text-blue-400" />
        <span className="font-semibold text-white">ClaudeContext Terminal</span>
      </div>
      
      {/* Spacer */}
      <div className="flex-1" />
      
      {/* Action Buttons */}
      <div className="flex items-center gap-2">
        <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors">
          <Folder className="w-4 h-4" />
        </button>
        <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors">
          <Server className="w-4 h-4" />
        </button>
        <button className="p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors">
          <Settings className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
};