import { invoke } from "@tauri-apps/api/core";

export interface TunnelInfo {
  id: string;
  tunnel_type: "local" | "remote" | "dynamic";
  description: string;
  active: boolean;
}

export interface TunnelConfig {
  id: number;
  session_id: number;
  tunnel_type: string;
  local_port: number | null;
  remote_host: string | null;
  remote_port: number | null;
  local_host: string | null;
  auto_start: boolean;
  name: string | null;
  sort_order: number;
}

// ── Active tunnel management ────────────────────────────────────────

export function createLocalForward(
  sessionId: string,
  localHost: string,
  localPort: number,
  remoteHost: string,
  remotePort: number
): Promise<string> {
  return invoke("create_local_forward", {
    sessionId,
    localHost,
    localPort,
    remoteHost,
    remotePort,
  });
}

export function createRemoteForward(
  sessionId: string,
  remotePort: number,
  localHost: string,
  localPort: number
): Promise<string> {
  return invoke("create_remote_forward", {
    sessionId,
    remotePort,
    localHost,
    localPort,
  });
}

export function createDynamicForward(
  sessionId: string,
  localHost: string,
  localPort: number
): Promise<string> {
  return invoke("create_dynamic_forward", {
    sessionId,
    localHost,
    localPort,
  });
}

export function stopTunnel(tunnelId: string): Promise<void> {
  return invoke("stop_tunnel", { tunnelId });
}

export function listTunnels(): Promise<TunnelInfo[]> {
  return invoke("list_tunnels");
}

// ── Saved tunnel configs ────────────────────────────────────────────

export function saveTunnelConfig(config: {
  sessionId: number;
  tunnelType: string;
  localPort?: number | null;
  remoteHost?: string | null;
  remotePort?: number | null;
  localHost?: string | null;
  autoStart: boolean;
  name?: string | null;
}): Promise<number> {
  return invoke("save_tunnel_config", {
    sessionId: config.sessionId,
    tunnelType: config.tunnelType,
    localPort: config.localPort ?? null,
    remoteHost: config.remoteHost ?? null,
    remotePort: config.remotePort ?? null,
    localHost: config.localHost ?? null,
    autoStart: config.autoStart,
    name: config.name ?? null,
  });
}

export function listTunnelConfigs(
  sessionId: number
): Promise<TunnelConfig[]> {
  return invoke("list_tunnel_configs", { sessionId });
}

export function deleteTunnelConfig(configId: number): Promise<boolean> {
  return invoke("delete_tunnel_config", { configId });
}
