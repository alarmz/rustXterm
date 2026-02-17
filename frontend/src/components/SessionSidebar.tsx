import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SessionInfo {
  id: number;
  name: string;
  protocol: string;
  host: string | null;
  port: number | null;
  username: string | null;
  is_favorite: boolean;
}

interface Props {
  visible: boolean;
  onConnectSession: (session: SessionInfo) => void;
  onNewSsh: () => void;
}

export default function SessionSidebar({
  visible,
  onConnectSession,
  onNewSsh,
}: Props) {
  const [sessions, setSessions] = useState<SessionInfo[]>([]);

  const loadSessions = useCallback(async () => {
    try {
      const list = await invoke<SessionInfo[]>("list_saved_sessions");
      setSessions(list);
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  }, []);

  useEffect(() => {
    if (visible) {
      loadSessions();
    }
  }, [visible, loadSessions]);

  const handleDelete = async (id: number, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke("delete_saved_session", { id });
      loadSessions();
    } catch (err) {
      console.error("Failed to delete session:", err);
    }
  };

  if (!visible) return null;

  const favorites = sessions.filter((s) => s.is_favorite);
  const others = sessions.filter((s) => !s.is_favorite);

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <span>Sessions</span>
        <button className="sidebar-add-btn" onClick={onNewSsh} title="New SSH Connection">
          +
        </button>
      </div>

      {favorites.length > 0 && (
        <div className="sidebar-section">
          <div className="sidebar-section-title">Favorites</div>
          {favorites.map((session) => (
            <SessionItem
              key={session.id}
              session={session}
              onConnect={onConnectSession}
              onDelete={handleDelete}
            />
          ))}
        </div>
      )}

      <div className="sidebar-section">
        <div className="sidebar-section-title">All Sessions</div>
        {others.length === 0 && favorites.length === 0 && (
          <div className="sidebar-empty">No saved sessions</div>
        )}
        {others.map((session) => (
          <SessionItem
            key={session.id}
            session={session}
            onConnect={onConnectSession}
            onDelete={handleDelete}
          />
        ))}
      </div>
    </div>
  );
}

function SessionItem({
  session,
  onConnect,
  onDelete,
}: {
  session: SessionInfo;
  onConnect: (s: SessionInfo) => void;
  onDelete: (id: number, e: React.MouseEvent) => void;
}) {
  return (
    <div
      className="sidebar-item"
      onDoubleClick={() => onConnect(session)}
      title={`${session.username || ""}@${session.host || ""}:${session.port || ""}`}
    >
      <span className="sidebar-item-icon">
        {session.protocol === "Ssh" ? "SSH" : "SH"}
      </span>
      <span className="sidebar-item-name">{session.name}</span>
      <button
        className="sidebar-item-delete"
        onClick={(e) => onDelete(session.id, e)}
        title="Delete"
      >
        x
      </button>
    </div>
  );
}
