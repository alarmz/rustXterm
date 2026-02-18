import { useState, useEffect } from "react";
import {
  sftpListTransfers,
  sftpCancelTransfer,
  type TransferInfo,
} from "../hooks/sftpApi";

function statusLabel(status: TransferInfo["status"]): string {
  switch (status) {
    case "pending":
      return "Pending";
    case "inprogress":
      return "In Progress";
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "cancelled":
      return "Cancelled";
  }
}

function progressPercent(info: TransferInfo): number {
  if (info.total_bytes === 0) return 100;
  return Math.round((info.transferred_bytes / info.total_bytes) * 100);
}

export default function TransferQueue() {
  const [transfers, setTransfers] = useState<TransferInfo[]>([]);

  useEffect(() => {
    // Poll transfers every 2 seconds
    const load = () => {
      sftpListTransfers()
        .then(setTransfers)
        .catch(() => {});
    };
    load();
    const interval = setInterval(load, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleCancel = async (id: string) => {
    await sftpCancelTransfer(id);
    const updated = await sftpListTransfers();
    setTransfers(updated);
  };

  if (transfers.length === 0) {
    return <div className="transfer-queue-empty">No transfers</div>;
  }

  return (
    <div className="transfer-queue">
      {transfers.map((t) => (
        <div key={t.id} className="transfer-item">
          <div className="transfer-info">
            <span className="transfer-direction">
              {t.direction === "download" ? "\u2193" : "\u2191"}
            </span>
            <span className="transfer-path">
              {t.source_path.split("/").pop()}
            </span>
            <span className={`transfer-status transfer-status-${t.status}`}>
              {statusLabel(t.status)}
            </span>
          </div>
          <div className="transfer-progress-bar">
            <div
              className="transfer-progress-fill"
              style={{ width: `${progressPercent(t)}%` }}
            />
          </div>
          {(t.status === "pending" || t.status === "inprogress") && (
            <button
              className="fb-btn-sm"
              onClick={() => handleCancel(t.id)}
              title="Cancel"
            >
              &#x2715;
            </button>
          )}
        </div>
      ))}
    </div>
  );
}
