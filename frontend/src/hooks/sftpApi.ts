import { invoke } from "@tauri-apps/api/core";

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified: number | null;
  permissions: number | null;
}

export interface TransferInfo {
  id: string;
  source_path: string;
  dest_path: string;
  direction: "download" | "upload";
  total_bytes: number;
  transferred_bytes: number;
  status: "pending" | "inprogress" | "completed" | "failed" | "cancelled";
}

// ── SFTP session lifecycle ──────────────────────────────────────────

export async function openSftp(sessionId: string): Promise<void> {
  return invoke("open_sftp", { sessionId });
}

export async function closeSftp(sessionId: string): Promise<void> {
  return invoke("close_sftp", { sessionId });
}

// ── SFTP file operations ────────────────────────────────────────────

export async function sftpListDir(
  sessionId: string,
  path: string
): Promise<FileEntry[]> {
  return invoke("sftp_list_dir", { sessionId, path });
}

export async function sftpStat(
  sessionId: string,
  path: string
): Promise<FileEntry> {
  return invoke("sftp_stat", { sessionId, path });
}

export async function sftpMkdir(
  sessionId: string,
  path: string
): Promise<void> {
  return invoke("sftp_mkdir", { sessionId, path });
}

export async function sftpRemove(
  sessionId: string,
  path: string,
  isDir: boolean
): Promise<void> {
  return invoke("sftp_remove", { sessionId, path, isDir });
}

export async function sftpRename(
  sessionId: string,
  oldPath: string,
  newPath: string
): Promise<void> {
  return invoke("sftp_rename", { sessionId, oldPath, newPath });
}

export async function sftpChmod(
  sessionId: string,
  path: string,
  mode: number
): Promise<void> {
  return invoke("sftp_chmod", { sessionId, path, mode });
}

export async function sftpReadFile(
  sessionId: string,
  path: string
): Promise<number[]> {
  return invoke("sftp_read_file", { sessionId, path });
}

export async function sftpWriteFile(
  sessionId: string,
  path: string,
  data: number[]
): Promise<void> {
  return invoke("sftp_write_file", { sessionId, path, data });
}

// ── File transfers ──────────────────────────────────────────────────

export async function sftpDownload(
  sessionId: string,
  remotePath: string,
  localPath: string
): Promise<string> {
  return invoke("sftp_download", { sessionId, remotePath, localPath });
}

export async function sftpUpload(
  sessionId: string,
  localPath: string,
  remotePath: string
): Promise<string> {
  return invoke("sftp_upload", { sessionId, localPath, remotePath });
}

export async function sftpCancelTransfer(transferId: string): Promise<boolean> {
  return invoke("sftp_cancel_transfer", { transferId });
}

export async function sftpListTransfers(): Promise<TransferInfo[]> {
  return invoke("sftp_list_transfers");
}

// ── Local filesystem ────────────────────────────────────────────────

export async function localListDir(path: string): Promise<FileEntry[]> {
  return invoke("local_list_dir", { path });
}

export async function localHomeDir(): Promise<string> {
  return invoke("local_home_dir");
}
