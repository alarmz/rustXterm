import { memo } from "react";
import type { Tab } from "../types";
import { getFirstLeaf } from "../hooks/paneUtils";

interface TabBarProps {
  tabs: Tab[];
  activeTabId: string | null;
  onSelectTab: (id: string) => void;
  onNewTab: () => void;
  onCloseTab: (id: string) => void;
}

export default memo(function TabBar({
  tabs,
  activeTabId,
  onSelectTab,
  onNewTab,
  onCloseTab,
}: TabBarProps) {
  return (
    <div className="tab-bar" role="tablist">
      {tabs.map((tab) => (
        <div
          key={tab.id}
          className={`tab ${tab.id === activeTabId ? "active" : ""}`}
          role="tab"
          tabIndex={0}
          aria-selected={tab.id === activeTabId}
          onClick={() => onSelectTab(tab.id)}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              onSelectTab(tab.id);
            }
          }}
        >
          <span className="tab-type-badge">
            {getFirstLeaf(tab.pane).sessionType === "ssh" ? "SSH" : "SH"}
          </span>
          <span>{tab.title}</span>
          <button
            className="tab-close"
            aria-label={`Close ${tab.title}`}
            onClick={(e) => {
              e.stopPropagation();
              onCloseTab(tab.id);
            }}
          >
            &times;
          </button>
        </div>
      ))}
      <button className="new-tab-btn" onClick={onNewTab} title="New Tab" aria-label="New Tab">
        +
      </button>
    </div>
  );
});
