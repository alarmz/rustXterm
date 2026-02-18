import { useState, useEffect, useCallback } from "react";
import { listTunnels, stopTunnel, type TunnelInfo } from "../hooks/tunnelApi";

interface Props {
  visible: boolean;
  onNewTunnel: () => void;
}

export default function TunnelList({ visible, onNewTunnel }: Props) {
  const [tunnels, setTunnels] = useState<TunnelInfo[]>([]);

  const refresh = useCallback(async () => {
    try {
      const list = await listTunnels();
      setTunnels(list);
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    if (!visible) return;
    refresh();
    const interval = setInterval(refresh, 3000);
    return () => clearInterval(interval);
  }, [visible, refresh]);

  const handleStop = useCallback(
    async (id: string) => {
      try {
        await stopTunnel(id);
        refresh();
      } catch (err) {
        console.error("Failed to stop tunnel:", err);
      }
    },
    [refresh]
  );

  if (!visible) return null;

  return (
    <div className="tunnel-list">
      <div className="tunnel-list-header">
        <span>Tunnels</span>
        <button className="sidebar-add-btn" onClick={onNewTunnel}>
          +
        </button>
      </div>
      {tunnels.length === 0 ? (
        <div className="tunnel-list-empty">No active tunnels</div>
      ) : (
        <div className="tunnel-list-entries">
          {tunnels.map((t) => (
            <div
              key={t.id}
              className={`tunnel-list-item${t.active ? "" : " tunnel-stopped"}`}
            >
              <span className="tunnel-type-badge">
                {t.tunnel_type === "local"
                  ? "L"
                  : t.tunnel_type === "remote"
                    ? "R"
                    : "D"}
              </span>
              <span className="tunnel-desc">{t.description}</span>
              <span
                className={`tunnel-status${t.active ? " tunnel-status-active" : " tunnel-status-inactive"}`}
              >
                {t.active ? "active" : "stopped"}
              </span>
              {t.active && (
                <button
                  className="tunnel-stop-btn"
                  onClick={() => handleStop(t.id)}
                >
                  Stop
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
