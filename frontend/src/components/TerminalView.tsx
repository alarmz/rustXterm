import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import "@xterm/xterm/css/xterm.css";

interface Props {
  sessionId: string;
  visible: boolean;
  isActive?: boolean;
}

interface PtyOutputPayload {
  session_id: string;
  data: number[];
}

export default function TerminalView({ sessionId, visible, isActive }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const visibleRef = useRef(visible);

  // Keep visibleRef in sync so the ResizeObserver callback always sees current value.
  visibleRef.current = visible;

  useEffect(() => {
    if (!containerRef.current) return;

    const term = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Cascadia Code', 'Fira Code', monospace",
      theme: {
        background: "#1e1e1e",
        foreground: "#d4d4d4",
        cursor: "#ffffff",
        selectionBackground: "#264f78",
      },
    });

    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.loadAddon(new WebLinksAddon());
    term.open(containerRef.current);
    fitAddon.fit();

    termRef.current = term;
    fitAddonRef.current = fitAddon;

    // Send user input to PTY
    const onDataDisposable = term.onData((data) => {
      const encoded = Array.from(new TextEncoder().encode(data));
      invoke("write_to_pty", { sessionId, data: encoded }).catch(() => {});
    });

    // Listen for PTY output — use abort flag to handle cleanup race.
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    listen<PtyOutputPayload>("pty-output", (event) => {
      if (event.payload.session_id === sessionId) {
        const bytes = new Uint8Array(event.payload.data);
        term.write(bytes);
      }
    }).then((fn) => {
      if (disposed) {
        fn(); // Already cleaned up, unlisten immediately
      } else {
        unlisten = fn;
      }
    });

    // Handle resize — read visibleRef (not stale `visible` closure)
    const resizeObserver = new ResizeObserver(() => {
      if (visibleRef.current) {
        fitAddon.fit();
        invoke("resize_pty", {
          sessionId,
          cols: term.cols,
          rows: term.rows,
        }).catch(() => {});
      }
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      disposed = true;
      resizeObserver.disconnect();
      onDataDisposable.dispose();
      if (unlisten) unlisten();
      term.dispose();
    };
  }, [sessionId]);

  // Re-fit when visibility changes
  useEffect(() => {
    let rafId: number | undefined;
    if (visible && fitAddonRef.current && termRef.current) {
      // Small delay to ensure container is visible before fitting
      rafId = requestAnimationFrame(() => {
        fitAddonRef.current?.fit();
        if (termRef.current) {
          invoke("resize_pty", {
            sessionId,
            cols: termRef.current.cols,
            rows: termRef.current.rows,
          }).catch(() => {});
        }
      });
    }
    return () => {
      if (rafId !== undefined) cancelAnimationFrame(rafId);
    };
  }, [visible, sessionId]);

  // Focus the terminal when this pane becomes active
  useEffect(() => {
    if (isActive && termRef.current) {
      termRef.current.focus();
    }
  }, [isActive]);

  return (
    <div
      ref={containerRef}
      className="terminal-view"
    />
  );
}
