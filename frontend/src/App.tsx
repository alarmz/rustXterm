import { useState, useCallback, useEffect } from "react";
import TabBar from "./components/TabBar";
import TerminalView from "./components/TerminalView";
import { useMenuEvents } from "./hooks/useMenuEvents";
import { spawnTerminal, closeTerminal } from "./hooks/useTerminal";

interface Tab {
  id: string;
  title: string;
  sessionId: string;
}

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [tabCounter, setTabCounter] = useState(0);

  const createNewTab = useCallback(async () => {
    try {
      const sessionId = await spawnTerminal(80, 24);
      const num = tabCounter + 1;
      setTabCounter(num);
      const newTab: Tab = {
        id: sessionId,
        title: `Terminal ${num}`,
        sessionId,
      };
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(sessionId);
    } catch (err) {
      console.error("Failed to spawn shell:", err);
    }
  }, [tabCounter]);

  const handleCloseTab = useCallback(
    async (tabId: string) => {
      try {
        await closeTerminal(tabId);
      } catch {
        // PTY may already be closed
      }
      setTabs((prev) => {
        const remaining = prev.filter((t) => t.id !== tabId);
        if (activeTabId === tabId && remaining.length > 0) {
          setActiveTabId(remaining[remaining.length - 1].id);
        } else if (remaining.length === 0) {
          setActiveTabId(null);
        }
        return remaining;
      });
    },
    [activeTabId]
  );

  const handleCloseActiveTab = useCallback(() => {
    if (activeTabId) {
      handleCloseTab(activeTabId);
    }
  }, [activeTabId, handleCloseTab]);

  // Listen for menu events
  useMenuEvents(createNewTab, handleCloseActiveTab);

  // Auto-create first tab on mount
  useEffect(() => {
    createNewTab();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="app">
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={setActiveTabId}
        onNewTab={createNewTab}
        onCloseTab={handleCloseTab}
      />
      <div className="terminal-container">
        {tabs.length === 0 ? (
          <div className="welcome">
            <h1>RustXterm</h1>
            <p>Press + or Ctrl+T to open a new terminal</p>
          </div>
        ) : (
          tabs.map((tab) => (
            <TerminalView
              key={tab.id}
              sessionId={tab.sessionId}
              visible={tab.id === activeTabId}
            />
          ))
        )}
      </div>
    </div>
  );
}

export default App;
