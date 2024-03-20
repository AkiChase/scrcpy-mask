import { invoke } from "@tauri-apps/api";

interface WindowInfo {
  hwnd: number;
  title: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

interface Device {
  id: string;
  status: string;
}

export async function getWindows(): Promise<WindowInfo[]> {
  return await invoke("get_windows");
}

export async function getWindowControls(hwnd: number): Promise<WindowInfo[]> {
  return await invoke("get_window_controls", { hwnd });
}

export async function adbDevices(): Promise<Device[]> {
  return await invoke("adb_devices");
}

export async function getScreenSize(id: string): Promise<[number, number]> {
  return await invoke("get_screen_size", { id });
}

export async function reverseServerPort(
  id: string,
  scid: string,
  port: number
): Promise<void> {
  return await invoke("reverse_server_port", { id, scid, port });
}

export async function pushServerFile(id: string): Promise<void> {
  return await invoke("push_server_file", { id });
}

export async function openSocketServer(port: number): Promise<void> {
  return await invoke("open_socket_server", { port });
}

export async function startScrcpyServer(
  id: string,
  scid: string
): Promise<void> {
  return await invoke("start_scrcpy_server", { id, scid });
}

export type { Device, WindowInfo };
