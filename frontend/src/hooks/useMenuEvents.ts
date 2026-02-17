import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export function useMenuEvents(
  onNewTab: () => void,
  onCloseTab: () => void
) {
  useEffect(() => {
    const unlisten = listen<string>("menu-event", (event) => {
      switch (event.payload) {
        case "new-tab":
          onNewTab();
          break;
        case "close-tab":
          onCloseTab();
          break;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onNewTab, onCloseTab]);
}
