import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

export function useMenuEvents(
  onNewTab: () => void,
  onCloseTab: () => void,
  onNewSsh: () => void,
  onToggleSidebar: () => void
) {
  // Store callbacks in refs so the listener never needs to re-register.
  const callbacks = useRef({ onNewTab, onCloseTab, onNewSsh, onToggleSidebar });
  callbacks.current = { onNewTab, onCloseTab, onNewSsh, onToggleSidebar };

  useEffect(() => {
    const unlistenPromise = listen<string>("menu-event", (event) => {
      switch (event.payload) {
        case "new-tab":
          callbacks.current.onNewTab();
          break;
        case "close-tab":
          callbacks.current.onCloseTab();
          break;
        case "new-ssh":
          callbacks.current.onNewSsh();
          break;
        case "toggle-sidebar":
          callbacks.current.onToggleSidebar();
          break;
      }
    });

    return () => {
      unlistenPromise.then((fn) => fn());
    };
  }, []);
}
