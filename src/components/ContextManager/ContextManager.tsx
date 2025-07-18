import React, { useEffect } from 'react';
import { useContext } from '../../contexts/ContextProvider';
import { FileText, RefreshCw, Settings } from 'lucide-react';

export const ContextManager: React.FC = () => {
  const { state, loadContextFiles, generateClaudeMd } = useContext();
  const { contextFiles, claudeMdContent, isLoading, error } = state;

  useEffect(() => {
    if (state.currentDirectory) {
      loadContextFiles();
    }
  }, [state.currentDirectory, loadContextFiles]);

  const handleRefresh = () => {
    loadContextFiles();
  };

  const handleGenerateClaudeMd = () => {
    generateClaudeMd();
  };

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <div className="text-gray-400">Loading context files...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-full flex items-center justify-center p-4">
        <div className="text-red-400">Error: {error}</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-700">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-lg font-semibold text-white">Context Files</h3>
          <div className="flex gap-2">
            <button
              onClick={handleRefresh}
              className="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
            </button>
            <button className="p-1 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors">
              <Settings className="w-4 h-4" />
            </button>
          </div>
        </div>
        
        <button
          onClick={handleGenerateClaudeMd}
          className="w-full px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded transition-colors"
        >
          Generate claude.md
        </button>
      </div>

      {/* File List */}
      <div className="flex-1 overflow-auto p-4">
        {contextFiles.length === 0 ? (
          <div className="text-center text-gray-400 py-8">
            <FileText className="w-12 h-12 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No markdown files found</p>
            <p className="text-xs mt-1">Create some .md files to get started</p>
          </div>
        ) : (
          <div className="space-y-2">
            {contextFiles.map((file) => (
              <div
                key={file.path}
                className="p-3 bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded cursor-pointer transition-colors"
              >
                <div className="flex items-center gap-2 mb-1">
                  <FileText className="w-4 h-4 text-blue-400" />
                  <span className="text-white text-sm font-medium">{file.name}</span>
                </div>
                <p className="text-gray-400 text-xs line-clamp-2">
                  {file.content.substring(0, 100)}
                  {file.content.length > 100 && '...'}
                </p>
                <div className="text-gray-500 text-xs mt-1">
                  Modified: {new Date(file.lastModified).toLocaleDateString()}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Claude.md Preview */}
      {claudeMdContent && (
        <div className="border-t border-gray-700 p-4">
          <h4 className="text-sm font-medium text-white mb-2">Claude.md Preview</h4>
          <div className="bg-gray-800 border border-gray-700 rounded p-3 text-xs text-gray-300 max-h-32 overflow-auto">
            <pre className="whitespace-pre-wrap">{claudeMdContent.content.substring(0, 300)}...</pre>
          </div>
        </div>
      )}
    </div>
  );
};