import { useState, useEffect, useCallback } from "react";
import {
  openSftp,
  sftpListDir,
  sftpMkdir,
  sftpRemove,
  sftpRename,
  sftpDownload,
  sftpUpload,
  localListDir,
  localHomeDir,
  type FileEntry,
} from "../hooks/sftpApi";
import TransferQueue from "./TransferQueue";

interface Props {
  sessionId: string;
  visible: boolean;
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

function formatDate(ts: number | null): string {
  if (ts === null) return "";
  return new Date(ts * 1000).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function FileBrowser({ sessionId, visible }: Props) {
  const [sftpReady, setSftpReady] = useState(false);
  const [remotePath, setRemotePath] = useState("/");
  const [localPath, setLocalPath] = useState("/");
  const [remoteEntries, setRemoteEntries] = useState<FileEntry[]>([]);
  const [localEntries, setLocalEntries] = useState<FileEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [showTransfers, setShowTransfers] = useState(false);
  const [activePanel, setActivePanel] = useState<"remote" | "local">("remote");

  // Initialize SFTP session
  useEffect(() => {
    if (!visible) return;
    let cancelled = false;

    (async () => {
      try {
        await openSftp(sessionId);
        if (!cancelled) setSftpReady(true);
      } catch (e) {
        if (!cancelled) setError(`Failed to open SFTP: ${e}`);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [sessionId, visible]);

  // Load remote directory
  const loadRemoteDir = useCallback(
    async (path: string) => {
      if (!sftpReady) return;
      try {
        setError(null);
        const entries = await sftpListDir(sessionId, path);
        entries.sort((a, b) => {
          if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        setRemoteEntries(entries);
        setRemotePath(path);
      } catch (e) {
        setError(`${e}`);
      }
    },
    [sessionId, sftpReady]
  );

  // Load local directory
  const loadLocalDir = useCallback(async (path: string) => {
    try {
      setError(null);
      const entries = await localListDir(path);
      entries.sort((a, b) => {
        if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
      setLocalEntries(entries);
      setLocalPath(path);
    } catch (e) {
      setError(`${e}`);
    }
  }, []);

  // Load initial directories
  useEffect(() => {
    if (sftpReady) {
      loadRemoteDir("/");
      localHomeDir()
        .then((home) => loadLocalDir(home))
        .catch(() => loadLocalDir("/"));
    }
  }, [sftpReady, loadRemoteDir, loadLocalDir]);

  const navigateUp = (path: string, isRemote: boolean) => {
    const parent = path.replace(/\/[^/]+\/?$/, "") || "/";
    if (isRemote) {
      loadRemoteDir(parent);
    } else {
      loadLocalDir(parent);
    }
  };

  const handleEntryClick = (entry: FileEntry, isRemote: boolean) => {
    if (entry.is_dir) {
      if (isRemote) {
        loadRemoteDir(entry.path);
      } else {
        loadLocalDir(entry.path);
      }
    }
  };

  const handleDownload = async (entry: FileEntry) => {
    try {
      const dest = `${localPath}/${entry.name}`.replace("//", "/");
      await sftpDownload(sessionId, entry.path, dest);
      loadLocalDir(localPath);
      setShowTransfers(true);
    } catch (e) {
      setError(`Download failed: ${e}`);
    }
  };

  const handleUpload = async (entry: FileEntry) => {
    try {
      const dest = `${remotePath}/${entry.name}`.replace("//", "/");
      await sftpUpload(sessionId, entry.path, dest);
      loadRemoteDir(remotePath);
      setShowTransfers(true);
    } catch (e) {
      setError(`Upload failed: ${e}`);
    }
  };

  const handleMkdir = async (isRemote: boolean) => {
    const name = prompt("Directory name:");
    if (!name) return;
    try {
      if (isRemote) {
        const fullPath = `${remotePath}/${name}`.replace("//", "/");
        await sftpMkdir(sessionId, fullPath);
        loadRemoteDir(remotePath);
      } else {
        // Local mkdir not yet implemented via command, skip for now
      }
    } catch (e) {
      setError(`Failed to create directory: ${e}`);
    }
  };

  const handleDelete = async (entry: FileEntry, isRemote: boolean) => {
    if (!confirm(`Delete ${entry.name}?`)) return;
    try {
      if (isRemote) {
        await sftpRemove(sessionId, entry.path, entry.is_dir);
        loadRemoteDir(remotePath);
      } else {
        // Local delete not yet implemented via command
      }
    } catch (e) {
      setError(`Failed to delete: ${e}`);
    }
  };

  const handleRename = async (entry: FileEntry) => {
    const newName = prompt("New name:", entry.name);
    if (!newName || newName === entry.name) return;
    try {
      const dir = entry.path.replace(/\/[^/]+$/, "");
      const newPath = `${dir}/${newName}`;
      await sftpRename(sessionId, entry.path, newPath);
      loadRemoteDir(remotePath);
    } catch (e) {
      setError(`Failed to rename: ${e}`);
    }
  };

  if (!visible) return null;

  if (!sftpReady) {
    return (
      <div className="file-browser">
        <div className="fb-loading">
          {error ? (
            <span className="fb-error">{error}</span>
          ) : (
            "Connecting SFTP..."
          )}
        </div>
      </div>
    );
  }

  const renderPanel = (
    entries: FileEntry[],
    currentPath: string,
    isRemote: boolean
  ) => (
    <div
      className={`fb-panel ${activePanel === (isRemote ? "remote" : "local") ? "fb-panel-active" : ""}`}
      onClick={() => setActivePanel(isRemote ? "remote" : "local")}
    >
      <div className="fb-panel-header">
        <span className="fb-panel-label">
          {isRemote ? "Remote" : "Local"}
        </span>
        <div className="fb-path-bar">
          <button
            className="fb-btn"
            onClick={(e) => {
              e.stopPropagation();
              navigateUp(currentPath, isRemote);
            }}
            title="Parent directory"
          >
            ..
          </button>
          <span className="fb-path">{currentPath}</span>
        </div>
        <div className="fb-panel-actions">
          {isRemote && (
            <button
              className="fb-btn"
              onClick={(e) => {
                e.stopPropagation();
                handleMkdir(isRemote);
              }}
              title="New directory"
            >
              +
            </button>
          )}
          <button
            className="fb-btn"
            onClick={(e) => {
              e.stopPropagation();
              isRemote
                ? loadRemoteDir(currentPath)
                : loadLocalDir(currentPath);
            }}
            title="Refresh"
          >
            &#x21bb;
          </button>
        </div>
      </div>
      <div className="fb-entries">
        {entries
          .filter((e) => !e.name.startsWith("."))
          .map((entry) => (
            <div
              key={entry.path}
              className="fb-entry"
              onDoubleClick={() => handleEntryClick(entry, isRemote)}
            >
              <span className="fb-icon">{entry.is_dir ? "\uD83D\uDCC1" : "\uD83D\uDCC4"}</span>
              <span className="fb-name">{entry.name}</span>
              <span className="fb-size">
                {entry.is_dir ? "" : formatSize(entry.size)}
              </span>
              <span className="fb-date">{formatDate(entry.modified)}</span>
              <div className="fb-entry-actions">
                {isRemote && !entry.is_dir && (
                  <button
                    className="fb-btn-sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDownload(entry);
                    }}
                    title="Download"
                  >
                    &#x2193;
                  </button>
                )}
                {!isRemote && !entry.is_dir && (
                  <button
                    className="fb-btn-sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleUpload(entry);
                    }}
                    title="Upload"
                  >
                    &#x2191;
                  </button>
                )}
                {isRemote && (
                  <>
                    <button
                      className="fb-btn-sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRename(entry);
                      }}
                      title="Rename"
                    >
                      &#x270E;
                    </button>
                    <button
                      className="fb-btn-sm fb-btn-danger"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(entry, isRemote);
                      }}
                      title="Delete"
                    >
                      &#x2715;
                    </button>
                  </>
                )}
              </div>
            </div>
          ))}
      </div>
    </div>
  );

  return (
    <div className="file-browser">
      {error && <div className="fb-error-bar">{error}</div>}
      <div className="fb-panels">
        {renderPanel(remoteEntries, remotePath, true)}
        {renderPanel(localEntries, localPath, false)}
      </div>
      <div className="fb-footer">
        <button
          className="fb-btn"
          onClick={() => setShowTransfers(!showTransfers)}
        >
          Transfers {showTransfers ? "\u25B2" : "\u25BC"}
        </button>
      </div>
      {showTransfers && <TransferQueue />}
    </div>
  );
}
