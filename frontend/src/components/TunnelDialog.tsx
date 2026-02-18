import { useState, useCallback } from "react";
import {
  createLocalForward,
  createRemoteForward,
  createDynamicForward,
} from "../hooks/tunnelApi";

interface Props {
  open: boolean;
  onClose: () => void;
  sessionId: string;
}

type TunnelType = "local" | "remote" | "dynamic";

export default function TunnelDialog({ open, onClose, sessionId }: Props) {
  const [tunnelType, setTunnelType] = useState<TunnelType>("local");
  const [localHost, setLocalHost] = useState("127.0.0.1");
  const [localPort, setLocalPort] = useState("");
  const [remoteHost, setRemoteHost] = useState("");
  const [remotePort, setRemotePort] = useState("");
  const [error, setError] = useState("");
  const [creating, setCreating] = useState(false);

  const resetForm = useCallback(() => {
    setLocalHost("127.0.0.1");
    setLocalPort("");
    setRemoteHost("");
    setRemotePort("");
    setError("");
    setCreating(false);
  }, []);

  const handleClose = useCallback(() => {
    resetForm();
    onClose();
  }, [resetForm, onClose]);

  const handleCreate = useCallback(async () => {
    setError("");
    setCreating(true);

    try {
      const lPort = parseInt(localPort, 10);
      const rPort = parseInt(remotePort, 10);

      if (tunnelType === "local") {
        if (!localPort || !remoteHost || !remotePort) {
          setError("Local port, remote host, and remote port are required");
          setCreating(false);
          return;
        }
        await createLocalForward(sessionId, localHost, lPort, remoteHost, rPort);
      } else if (tunnelType === "remote") {
        if (!remotePort || !localPort) {
          setError("Remote port and local port are required");
          setCreating(false);
          return;
        }
        await createRemoteForward(sessionId, rPort, localHost, lPort);
      } else {
        if (!localPort) {
          setError("Local port is required");
          setCreating(false);
          return;
        }
        await createDynamicForward(sessionId, localHost, lPort);
      }

      handleClose();
    } catch (err) {
      setError(String(err));
      setCreating(false);
    }
  }, [tunnelType, localHost, localPort, remoteHost, remotePort, sessionId, handleClose]);

  if (!open) return null;

  return (
    <div className="dialog-overlay" onMouseDown={handleClose}>
      <div className="connect-dialog" onMouseDown={(e) => e.stopPropagation()}>
        <h2>New Tunnel</h2>

        {error && <div className="connect-error">{error}</div>}

        <div className="form-row">
          <label>Type</label>
          <div className="tunnel-type-selector">
            {(["local", "remote", "dynamic"] as TunnelType[]).map((t) => (
              <button
                key={t}
                className={`tunnel-type-btn${tunnelType === t ? " active" : ""}`}
                onClick={() => setTunnelType(t)}
              >
                {t === "local" ? "-L Local" : t === "remote" ? "-R Remote" : "-D Dynamic"}
              </button>
            ))}
          </div>
        </div>

        {tunnelType === "local" && (
          <>
            <div className="form-row">
              <label>Local Host</label>
              <input
                value={localHost}
                onChange={(e) => setLocalHost(e.target.value)}
                placeholder="127.0.0.1"
              />
            </div>
            <div className="form-row">
              <label>Local Port</label>
              <input
                value={localPort}
                onChange={(e) => setLocalPort(e.target.value)}
                placeholder="8080"
                type="number"
              />
            </div>
            <div className="form-row">
              <label>Remote Host</label>
              <input
                value={remoteHost}
                onChange={(e) => setRemoteHost(e.target.value)}
                placeholder="db.internal"
              />
            </div>
            <div className="form-row">
              <label>Remote Port</label>
              <input
                value={remotePort}
                onChange={(e) => setRemotePort(e.target.value)}
                placeholder="5432"
                type="number"
              />
            </div>
          </>
        )}

        {tunnelType === "remote" && (
          <>
            <div className="form-row">
              <label>Remote Port (server listens on)</label>
              <input
                value={remotePort}
                onChange={(e) => setRemotePort(e.target.value)}
                placeholder="8080"
                type="number"
              />
            </div>
            <div className="form-row">
              <label>Local Host</label>
              <input
                value={localHost}
                onChange={(e) => setLocalHost(e.target.value)}
                placeholder="127.0.0.1"
              />
            </div>
            <div className="form-row">
              <label>Local Port</label>
              <input
                value={localPort}
                onChange={(e) => setLocalPort(e.target.value)}
                placeholder="3000"
                type="number"
              />
            </div>
          </>
        )}

        {tunnelType === "dynamic" && (
          <>
            <div className="form-row">
              <label>Local Host</label>
              <input
                value={localHost}
                onChange={(e) => setLocalHost(e.target.value)}
                placeholder="127.0.0.1"
              />
            </div>
            <div className="form-row">
              <label>Local Port (SOCKS5 proxy)</label>
              <input
                value={localPort}
                onChange={(e) => setLocalPort(e.target.value)}
                placeholder="1080"
                type="number"
              />
            </div>
          </>
        )}

        <div className="tunnel-description">
          {tunnelType === "local" &&
            localPort &&
            remoteHost &&
            remotePort &&
            `ssh -L ${localHost}:${localPort}:${remoteHost}:${remotePort}`}
          {tunnelType === "remote" &&
            remotePort &&
            localPort &&
            `ssh -R ${remotePort}:${localHost}:${localPort}`}
          {tunnelType === "dynamic" &&
            localPort &&
            `ssh -D ${localHost}:${localPort}`}
        </div>

        <div className="form-actions">
          <button className="btn-cancel" onClick={handleClose}>
            Cancel
          </button>
          <button
            className="btn-connect"
            onClick={handleCreate}
            disabled={creating}
          >
            {creating ? "Creating..." : "Create Tunnel"}
          </button>
        </div>
      </div>
    </div>
  );
}
