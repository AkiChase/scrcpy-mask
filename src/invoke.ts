import { invoke } from '@tauri-apps/api/core';

interface Device {
  id: string;
  status: string;
}

export async function adbDevices(): Promise<Device[]> {
  return await invoke("adb_devices");
}

export async function forwardServerPort(
  id: string,
  scid: string,
  port: number
): Promise<void> {
  return await invoke("forward_server_port", { id, scid, port });
}

export async function pushServerFile(id: string): Promise<void> {
  return await invoke("push_server_file", { id });
}

export async function startScrcpyServer(
  id: string,
  scid: string,
  address: string
): Promise<void> {
  return await invoke("start_scrcpy_server", { id, scid, address });
}

export type { Device };
