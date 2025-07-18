import { SplitView } from './components/Layout/SplitView';
import { ContextProvider } from './contexts/ContextProvider';
import { KanbanProvider } from './contexts/KanbanProvider';
import './styles/globals.css';

function App() {
  return (
    <ContextProvider>
      <KanbanProvider>
        <div className="h-screen w-screen bg-gray-900 text-gray-100">
          <SplitView />
        </div>
      </KanbanProvider>
    </ContextProvider>
  );
}

export default App;
