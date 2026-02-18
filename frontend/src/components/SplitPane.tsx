import type { PaneNode } from "../types";
import DragHandle from "./DragHandle";
import TerminalView from "./TerminalView";

interface Props {
  node: PaneNode;
  onNodeChange: (node: PaneNode) => void;
  activePaneId: string | null;
  onPaneClick: (paneId: string) => void;
  tabVisible: boolean;
}

export default function SplitPane({
  node,
  onNodeChange,
  activePaneId,
  onPaneClick,
  tabVisible,
}: Props) {
  if (node.type === "leaf") {
    return (
      <div
        className={`pane-leaf${node.id === activePaneId ? " pane-active" : ""}`}
        onMouseDown={() => onPaneClick(node.id)}
      >
        <TerminalView
          sessionId={node.sessionId}
          visible={tabVisible}
          isActive={tabVisible && node.id === activePaneId}
        />
      </div>
    );
  }

  return (
    <div className={`pane-split pane-split-${node.direction}`}>
      <div className="pane-child" style={{ flex: node.ratio }}>
        <SplitPane
          node={node.first}
          onNodeChange={(n) => onNodeChange({ ...node, first: n })}
          activePaneId={activePaneId}
          onPaneClick={onPaneClick}
          tabVisible={tabVisible}
        />
      </div>
      <DragHandle
        direction={node.direction}
        onRatioChange={(ratio) => onNodeChange({ ...node, ratio })}
      />
      <div className="pane-child" style={{ flex: 1 - node.ratio }}>
        <SplitPane
          node={node.second}
          onNodeChange={(n) => onNodeChange({ ...node, second: n })}
          activePaneId={activePaneId}
          onPaneClick={onPaneClick}
          tabVisible={tabVisible}
        />
      </div>
    </div>
  );
}
