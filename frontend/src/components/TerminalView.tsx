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
}

interface PtyOutputPayload {
  session_id: string;
  data: number[];
}

export default function TerminalView({ sessionId, visible }: Props) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);

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
      invoke("write_to_pty", { sessionId, data: encoded });
    });

    // Listen for PTY output
    let unlisten: UnlistenFn | undefined;
    listen<PtyOutputPayload>("pty-output", (event) => {
      if (event.payload.session_id === sessionId) {
        const bytes = new Uint8Array(event.payload.data);
        term.write(bytes);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      if (visible) {
        fitAddon.fit();
        invoke("resize_pty", {
          sessionId,
          cols: term.cols,
          rows: term.rows,
        });
      }
    });
    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
      onDataDisposable.dispose();
      if (unlisten) unlisten();
      term.dispose();
    };
  }, [sessionId]);

  // Re-fit when visibility changes
  useEffect(() => {
    if (visible && fitAddonRef.current && termRef.current) {
      // Small delay to ensure container is visible before fitting
      requestAnimationFrame(() => {
        fitAddonRef.current?.fit();
        if (termRef.current) {
          invoke("resize_pty", {
            sessionId,
            cols: termRef.current.cols,
            rows: termRef.current.rows,
          });
          termRef.current.focus();
        }
      });
    }
  }, [visible, sessionId]);

  return (
    <div
      ref={containerRef}
      className="terminal-view"
      style={{ display: visible ? "block" : "none" }}
    />
  );
}
