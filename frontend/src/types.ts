// ── Pane tree model ──────────────────────────────────────────

export interface PaneLeaf {
  type: "leaf";
  id: string;
  sessionId: string;
  sessionType: "local" | "ssh";
}

export interface PaneSplit {
  type: "split";
  direction: "horizontal" | "vertical";
  ratio: number;
  first: PaneNode;
  second: PaneNode;
}

export type PaneNode = PaneLeaf | PaneSplit;

// ── Tab model ────────────────────────────────────────────────

export interface Tab {
  id: string;
  title: string;
  pane: PaneNode;
}
