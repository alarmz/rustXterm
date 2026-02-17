import { useState, useEffect } from "react";

interface Props {
  open: boolean;
  onClose: () => void;
  onConnect: (host: string, port: number, username: string, password: string) => Promise<void> | void;
}

export default function ConnectDialog({ open, onClose, onConnect }: Props) {
  const [host, setHost] = useState("");
  const [port, setPort] = useState("22");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [connecting, setConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Reset form state when dialog opens.
  useEffect(() => {
    if (open) {
      setHost("");
      setPort("22");
      setUsername("");
      setPassword("");
      setConnecting(false);
      setError(null);
    }
  }, [open]);

  if (!open) return null;

  const portNum = parseInt(port) || 0;
  const portValid = portNum >= 1 && portNum <= 65535;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!host || !username || !portValid) return;
    setConnecting(true);
    setError(null);
    try {
      await onConnect(host, portNum, username, password);
      setHost("");
      setPort("22");
      setUsername("");
      setPassword("");
    } catch (err) {
      setError(String(err));
    } finally {
      setConnecting(false);
    }
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="connect-dialog" onClick={(e) => e.stopPropagation()}>
        <h2>SSH Connection</h2>
        {error && <div className="connect-error">{error}</div>}
        <form onSubmit={handleSubmit}>
          <div className="form-row">
            <label htmlFor="host">Host</label>
            <input
              id="host"
              type="text"
              value={host}
              onChange={(e) => setHost(e.target.value)}
              placeholder="hostname or IP"
              autoFocus
            />
          </div>
          <div className="form-row">
            <label htmlFor="port">Port</label>
            <input
              id="port"
              type="number"
              min={1}
              max={65535}
              value={port}
              onChange={(e) => setPort(e.target.value)}
              placeholder="22"
            />
          </div>
          <div className="form-row">
            <label htmlFor="username">Username</label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="root"
            />
          </div>
          <div className="form-row">
            <label htmlFor="password">Password</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </div>
          <div className="form-actions">
            <button type="button" className="btn-cancel" onClick={onClose}>
              Cancel
            </button>
            <button
              type="submit"
              className="btn-connect"
              disabled={!host || !username || !portValid || connecting}
            >
              {connecting ? "Connecting..." : "Connect"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
