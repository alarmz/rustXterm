import { useState, useCallback, useEffect, useRef } from "react";
import TabBar from "./components/TabBar";
import TerminalView from "./components/TerminalView";
import ConnectDialog from "./components/ConnectDialog";
import SessionSidebar from "./components/SessionSidebar";
import { useMenuEvents } from "./hooks/useMenuEvents";
import { spawnTerminal, connectSsh, closeTerminal } from "./hooks/terminalApi";
import type { Tab } from "./types";

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [showConnectDialog, setShowConnectDialog] = useState(false);
  const [showSidebar, setShowSidebar] = useState(false);

  // Use a ref for tabCounter to avoid stale closures in callbacks.
  const tabCounterRef = useRef(0);

  // Keep activeTabId accessible via ref for closures that shouldn't re-create.
  const activeTabIdRef = useRef(activeTabId);
  activeTabIdRef.current = activeTabId;

  const createNewTab = useCallback(async () => {
    try {
      const sessionId = await spawnTerminal(80, 24);
      tabCounterRef.current += 1;
      const num = tabCounterRef.current;
      const newTab: Tab = {
        id: sessionId,
        title: `Terminal ${num}`,
        sessionId,
        type: "local",
      };
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(sessionId);
    } catch (err) {
      console.error("Failed to spawn shell:", err);
    }
  }, []);

  const handleSshConnect = useCallback(
    async (host: string, port: number, username: string, password: string) => {
      const sessionId = await connectSsh(host, port, username, password, 80, 24);
      const newTab: Tab = {
        id: sessionId,
        title: `${username}@${host}`,
        sessionId,
        type: "ssh",
      };
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(sessionId);
      setShowConnectDialog(false);
    },
    []
  );

  const handleCloseTab = useCallback(async (tabId: string) => {
    try {
      await closeTerminal(tabId);
    } catch {
      // Session may already be closed
    }
    setTabs((prev) => {
      const remaining = prev.filter((t) => t.id !== tabId);
      // Use ref to avoid depending on activeTabId in the dep array.
      if (activeTabIdRef.current === tabId) {
        if (remaining.length > 0) {
          setActiveTabId(remaining[remaining.length - 1].id);
        } else {
          setActiveTabId(null);
        }
      }
      return remaining;
    });
  }, []);

  const handleCloseActiveTab = useCallback(() => {
    const current = activeTabIdRef.current;
    if (current) {
      handleCloseTab(current);
    }
  }, [handleCloseTab]);

  const handleNewSsh = useCallback(() => {
    setShowConnectDialog(true);
  }, []);

  const handleToggleSidebar = useCallback(() => {
    setShowSidebar((prev) => !prev);
  }, []);

  const handleConnectSavedSession = useCallback(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    async (session: any) => {
      if (session.protocol === "Ssh" && session.host) {
        // For saved SSH sessions, we'd need to retrieve the password from credentials.
        // For now, open the connect dialog pre-filled.
        setShowConnectDialog(true);
      }
    },
    []
  );

  // Listen for menu events
  useMenuEvents(createNewTab, handleCloseActiveTab, handleNewSsh, handleToggleSidebar);

  // Auto-create first tab on mount
  useEffect(() => {
    createNewTab();
  }, [createNewTab]);

  return (
    <div className="app">
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={setActiveTabId}
        onNewTab={createNewTab}
        onCloseTab={handleCloseTab}
      />
      <div className="main-content">
        <SessionSidebar
          visible={showSidebar}
          onConnectSession={handleConnectSavedSession}
          onNewSsh={handleNewSsh}
        />
        <div className="terminal-container">
          {tabs.length === 0 ? (
            <div className="welcome">
              <h1>RustXterm</h1>
              <p>Press + or Ctrl+T to open a new terminal</p>
              <p>Press Ctrl+Shift+S to open an SSH connection</p>
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
      <ConnectDialog
        open={showConnectDialog}
        onClose={() => setShowConnectDialog(false)}
        onConnect={handleSshConnect}
      />
    </div>
  );
}

export default App;
