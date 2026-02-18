import type { PaneNode, PaneLeaf } from "../types";

/** Create a new leaf pane. */
export function createLeaf(
  sessionId: string,
  sessionType: "local" | "ssh"
): PaneLeaf {
  return {
    type: "leaf",
    id: crypto.randomUUID(),
    sessionId,
    sessionType,
  };
}

/** Count all leaf panes in the tree. */
export function countLeaves(node: PaneNode): number {
  if (node.type === "leaf") return 1;
  return countLeaves(node.first) + countLeaves(node.second);
}

/** Get the first (leftmost/topmost) leaf in the tree. */
export function getFirstLeaf(node: PaneNode): PaneLeaf {
  if (node.type === "leaf") return node;
  return getFirstLeaf(node.first);
}

/** Get all leaf nodes in left-to-right order. */
export function getAllLeaves(node: PaneNode): PaneLeaf[] {
  if (node.type === "leaf") return [node];
  return [...getAllLeaves(node.first), ...getAllLeaves(node.second)];
}

/** Find a leaf by pane ID. */
export function findLeaf(
  node: PaneNode,
  paneId: string
): PaneLeaf | null {
  if (node.type === "leaf") return node.id === paneId ? node : null;
  return findLeaf(node.first, paneId) ?? findLeaf(node.second, paneId);
}

/**
 * Split a leaf pane into two. The original becomes the first child
 * and a new leaf becomes the second. Returns null if the target is
 * not found or max panes is reached.
 */
export function splitPane(
  root: PaneNode,
  targetPaneId: string,
  direction: "horizontal" | "vertical",
  newLeaf: PaneLeaf,
  maxPanes = 4
): PaneNode | null {
  if (countLeaves(root) >= maxPanes) return null;

  function walk(node: PaneNode): PaneNode | null {
    if (node.type === "leaf") {
      if (node.id === targetPaneId) {
        return {
          type: "split",
          direction,
          ratio: 0.5,
          first: node,
          second: newLeaf,
        };
      }
      return null;
    }
    const newFirst = walk(node.first);
    if (newFirst) return { ...node, first: newFirst };
    const newSecond = walk(node.second);
    if (newSecond) return { ...node, second: newSecond };
    return null;
  }

  return walk(root);
}

/**
 * Close a leaf pane. The sibling takes over the parent split.
 * Returns null if the root is the target (last pane).
 * Returns the same root reference if the target is not found.
 */
export function closePane(
  root: PaneNode,
  targetPaneId: string
): PaneNode | null {
  if (root.type === "leaf") {
    return root.id === targetPaneId ? null : root;
  }

  if (root.first.type === "leaf" && root.first.id === targetPaneId) {
    return root.second;
  }
  if (root.second.type === "leaf" && root.second.id === targetPaneId) {
    return root.first;
  }

  const newFirst = closePane(root.first, targetPaneId);
  if (newFirst !== root.first) {
    return newFirst === null ? root.second : { ...root, first: newFirst };
  }
  const newSecond = closePane(root.second, targetPaneId);
  if (newSecond !== root.second) {
    return newSecond === null ? root.first : { ...root, second: newSecond };
  }
  return root;
}

/** Get the next leaf after the given one (wraps around). */
export function getNextLeaf(
  root: PaneNode,
  currentPaneId: string
): PaneLeaf | null {
  const leaves = getAllLeaves(root);
  const idx = leaves.findIndex((l) => l.id === currentPaneId);
  if (idx === -1 || leaves.length <= 1) return null;
  return leaves[(idx + 1) % leaves.length];
}

/** Get the previous leaf before the given one (wraps around). */
export function getPrevLeaf(
  root: PaneNode,
  currentPaneId: string
): PaneLeaf | null {
  const leaves = getAllLeaves(root);
  const idx = leaves.findIndex((l) => l.id === currentPaneId);
  if (idx === -1 || leaves.length <= 1) return null;
  return leaves[(idx - 1 + leaves.length) % leaves.length];
}
