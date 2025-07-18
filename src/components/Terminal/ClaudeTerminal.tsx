import React, { useEffect, useRef } from 'react';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { SearchAddon } from '@xterm/addon-search';

export const ClaudeTerminal: React.FC = () => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const fitAddon = useRef<FitAddon>(new FitAddon());

  useEffect(() => {
    if (!terminalRef.current || terminal.current) return;

    // Create terminal instance
    terminal.current = new Terminal({
      theme: {
        background: '#0d1117',
        foreground: '#c9d1d9',
        cursor: '#c9d1d9',
        selectionBackground: '#264f78',
        black: '#484f58',
        red: '#ff7b72',
        green: '#7ee787',
        yellow: '#f0883e',
        blue: '#58a6ff',
        magenta: '#bc8cff',
        cyan: '#39c5cf',
        white: '#b1bac4',
        brightBlack: '#6e7681',
        brightRed: '#ff7b72',
        brightGreen: '#7ee787',
        brightYellow: '#f0883e',
        brightBlue: '#58a6ff',
        brightMagenta: '#bc8cff',
        brightCyan: '#39c5cf',
        brightWhite: '#f0f6fc',
      },
      fontFamily: 'JetBrains Mono, Monaco, Menlo, monospace',
      fontSize: 14,
      lineHeight: 1.4,
      cursorBlink: true,
      allowTransparency: true,
    });

    // Load addons
    terminal.current.loadAddon(fitAddon.current);
    terminal.current.loadAddon(new WebLinksAddon());
    terminal.current.loadAddon(new SearchAddon());

    // Open terminal
    terminal.current.open(terminalRef.current);
    fitAddon.current.fit();

    // Welcome message
    terminal.current.writeln('Welcome to ClaudeContext Terminal');
    terminal.current.writeln('Initializing Claude CLI integration...');
    terminal.current.writeln('');
    terminal.current.write('$ ');

    // Handle resize
    const handleResize = () => {
      if (terminal.current && fitAddon.current) {
        fitAddon.current.fit();
      }
    };

    window.addEventListener('resize', handleResize);

    // Handle input
    terminal.current.onData((data) => {
      if (terminal.current) {
        // Echo the input for now
        terminal.current.write(data);
        
        // Handle enter key
        if (data === '\r' || data === '\n') {
          terminal.current.writeln('');
          terminal.current.write('$ ');
        }
      }
    });

    return () => {
      window.removeEventListener('resize', handleResize);
      if (terminal.current) {
        terminal.current.dispose();
        terminal.current = null;
      }
    };
  }, []);

  return (
    <div className="h-full flex flex-col">
      {/* Terminal Header */}
      <div className="h-12 bg-gray-800 border-b border-gray-700 flex items-center justify-between px-4">
        <div className="flex items-center gap-2">
          <div className="flex gap-2">
            <div className="w-3 h-3 bg-red-500 rounded-full"></div>
            <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
            <div className="w-3 h-3 bg-green-500 rounded-full"></div>
          </div>
          <span className="text-gray-400 text-sm ml-2">Claude CLI</span>
        </div>
        
        <div className="flex items-center gap-2">
          <div className="text-xs text-gray-400">Session: Active</div>
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
        </div>
      </div>

      {/* Terminal Content */}
      <div 
        ref={terminalRef}
        className="flex-1 terminal-container"
      />
    </div>
  );
};