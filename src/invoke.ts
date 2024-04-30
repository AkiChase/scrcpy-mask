import { invoke } from "@tauri-apps/api/core";

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

export async function getDeviceScreenSize(
  id: string
): Promise<[number, number]> {
  return await invoke("get_device_screen_size", { id });
}

export async function adbConnect(address: string): Promise<string> {
  return await invoke("adb_connect", { address });
}

export async function loadDefaultKeyconfig(): Promise<string> {
  return await invoke("load_default_keyconfig");
}

export type { Device };
