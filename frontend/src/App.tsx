import { useState, useCallback, useEffect, useRef } from "react";
import TabBar from "./components/TabBar";
import SplitPaneContainer from "./components/SplitPaneContainer";
import ConnectDialog from "./components/ConnectDialog";
import SessionSidebar from "./components/SessionSidebar";
import { useMenuEvents } from "./hooks/useMenuEvents";
import { spawnTerminal, connectSsh, closeTerminal } from "./hooks/terminalApi";
import {
  createLeaf,
  splitPane,
  closePane,
  getAllLeaves,
  getFirstLeaf,
  getNextLeaf,
} from "./hooks/paneUtils";
import type { Tab, PaneNode } from "./types";

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [activePaneId, setActivePaneId] = useState<string | null>(null);
  const [showConnectDialog, setShowConnectDialog] = useState(false);
  const [showSidebar, setShowSidebar] = useState(false);

  const tabCounterRef = useRef(0);
  const activeTabIdRef = useRef(activeTabId);
  activeTabIdRef.current = activeTabId;
  const activePaneIdRef = useRef(activePaneId);
  activePaneIdRef.current = activePaneId;
  const tabsRef = useRef(tabs);
  tabsRef.current = tabs;

  const createNewTab = useCallback(async () => {
    try {
      const sessionId = await spawnTerminal(80, 24);
      tabCounterRef.current += 1;
      const num = tabCounterRef.current;
      const leaf = createLeaf(sessionId, "local");
      const newTab: Tab = {
        id: leaf.id,
        title: `Terminal ${num}`,
        pane: leaf,
      };
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(newTab.id);
      setActivePaneId(leaf.id);
    } catch (err) {
      console.error("Failed to spawn shell:", err);
    }
  }, []);

  const handleSshConnect = useCallback(
    async (host: string, port: number, username: string, password: string) => {
      const sessionId = await connectSsh(host, port, username, password, 80, 24);
      const leaf = createLeaf(sessionId, "ssh");
      const newTab: Tab = {
        id: leaf.id,
        title: `${username}@${host}`,
        pane: leaf,
      };
      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(newTab.id);
      setActivePaneId(leaf.id);
      setShowConnectDialog(false);
    },
    []
  );

  const handleCloseTab = useCallback(async (tabId: string) => {
    const tab = tabsRef.current.find((t) => t.id === tabId);
    if (tab) {
      for (const leaf of getAllLeaves(tab.pane)) {
        closeTerminal(leaf.sessionId).catch(() => {});
      }
    }
    setTabs((prev) => {
      const remaining = prev.filter((t) => t.id !== tabId);
      if (activeTabIdRef.current === tabId) {
        if (remaining.length > 0) {
          const newActive = remaining[remaining.length - 1];
          setActiveTabId(newActive.id);
          setActivePaneId(getFirstLeaf(newActive.pane).id);
        } else {
          setActiveTabId(null);
          setActivePaneId(null);
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

  const handleSplitPane = useCallback(
    async (direction: "horizontal" | "vertical") => {
      const tabId = activeTabIdRef.current;
      const paneId = activePaneIdRef.current;
      if (!tabId || !paneId) return;

      try {
        const sessionId = await spawnTerminal(80, 24);
        const newLeaf = createLeaf(sessionId, "local");

        setTabs((prev) =>
          prev.map((tab) => {
            if (tab.id !== tabId) return tab;
            const newPane = splitPane(tab.pane, paneId, direction, newLeaf);
            if (!newPane) return tab;
            return { ...tab, pane: newPane };
          })
        );
        setActivePaneId(newLeaf.id);
      } catch (err) {
        console.error("Failed to split pane:", err);
      }
    },
    []
  );

  const handleClosePane = useCallback(async () => {
    const tabId = activeTabIdRef.current;
    const paneId = activePaneIdRef.current;
    if (!tabId || !paneId) return;

    const tab = tabsRef.current.find((t) => t.id === tabId);
    if (!tab) return;

    const leaf = getAllLeaves(tab.pane).find((l) => l.id === paneId);
    if (leaf) {
      closeTerminal(leaf.sessionId).catch(() => {});
    }

    const newPane = closePane(tab.pane, paneId);

    if (newPane === null) {
      // Last pane — close the entire tab
      setTabs((prev) => {
        const remaining = prev.filter((t) => t.id !== tabId);
        if (remaining.length > 0) {
          const newActive = remaining[remaining.length - 1];
          setActiveTabId(newActive.id);
          setActivePaneId(getFirstLeaf(newActive.pane).id);
        } else {
          setActiveTabId(null);
          setActivePaneId(null);
        }
        return remaining;
      });
      return;
    }

    if (newPane === tab.pane) return; // target not found

    const nextLeaf = getNextLeaf(newPane, paneId) ?? getFirstLeaf(newPane);
    setActivePaneId(nextLeaf.id);
    setTabs((prev) =>
      prev.map((t) => (t.id === tabId ? { ...t, pane: newPane } : t))
    );
  }, []);

  const handlePaneChange = useCallback(
    (tabId: string, newPane: PaneNode) => {
      setTabs((prev) =>
        prev.map((tab) => (tab.id === tabId ? { ...tab, pane: newPane } : tab))
      );
    },
    []
  );

  const handleTabSelect = useCallback((tabId: string) => {
    setActiveTabId(tabId);
    const tab = tabsRef.current.find((t) => t.id === tabId);
    if (tab) {
      setActivePaneId(getFirstLeaf(tab.pane).id);
    }
  }, []);

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
        setShowConnectDialog(true);
      }
    },
    []
  );

  const handleSplitHorizontal = useCallback(() => {
    handleSplitPane("horizontal");
  }, [handleSplitPane]);

  const handleSplitVertical = useCallback(() => {
    handleSplitPane("vertical");
  }, [handleSplitPane]);

  // Listen for menu events
  useMenuEvents(
    createNewTab,
    handleCloseActiveTab,
    handleNewSsh,
    handleToggleSidebar,
    handleSplitHorizontal,
    handleSplitVertical,
    handleClosePane
  );

  // Auto-create first tab on mount
  useEffect(() => {
    createNewTab();
  }, [createNewTab]);

  return (
    <div className="app">
      <TabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onSelectTab={handleTabSelect}
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
              <div
                key={tab.id}
                className="tab-panes"
                style={{ display: tab.id === activeTabId ? "flex" : "none" }}
              >
                <SplitPaneContainer
                  pane={tab.pane}
                  onPaneChange={(newPane) => handlePaneChange(tab.id, newPane)}
                  activePaneId={activePaneId}
                  onPaneClick={setActivePaneId}
                  tabVisible={tab.id === activeTabId}
                />
              </div>
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
