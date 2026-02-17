import { invoke } from "@tauri-apps/api/core";

export async function spawnTerminal(
  cols: number = 80,
  rows: number = 24
): Promise<string> {
  return invoke<string>("spawn_shell", { cols, rows });
}

export async function closeTerminal(sessionId: string): Promise<void> {
  return invoke("close_pty", { sessionId });
}
