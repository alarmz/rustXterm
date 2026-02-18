import type { PaneNode } from "../types";
import SplitPane from "./SplitPane";

interface Props {
  pane: PaneNode;
  onPaneChange: (pane: PaneNode) => void;
  activePaneId: string | null;
  onPaneClick: (paneId: string) => void;
  tabVisible: boolean;
}

export default function SplitPaneContainer({
  pane,
  onPaneChange,
  activePaneId,
  onPaneClick,
  tabVisible,
}: Props) {
  return (
    <div className="split-pane-container">
      <SplitPane
        node={pane}
        onNodeChange={onPaneChange}
        activePaneId={activePaneId}
        onPaneClick={onPaneClick}
        tabVisible={tabVisible}
      />
    </div>
  );
}
