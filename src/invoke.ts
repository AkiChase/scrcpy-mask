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

async function getWindows(): Promise<WindowInfo[]> {
  return await invoke("get_windows");
}

async function getWindowControls(hwnd: number): Promise<WindowInfo[]> {
  return await invoke("get_window_controls", { hwnd });
}

async function adbDevices(): Promise<Device[]> {
  return await invoke("adb_devices");
}

async function reverseServerPort(
  id: string,
  scid: string,
  port: number
): Promise<void> {
  return await invoke("reverse_server_port", { id, scid, port });
}

async function pushServerFile(id: string): Promise<void> {
  return await invoke("push_server_file", { id });
}

async function openSocketServer(port: number): Promise<void> {
  return await invoke("open_socket_server", { port });
}

async function closeSocketServer(): Promise<void> {
  return await invoke("close_socket_server");
}

async function startScrcpyServer(id: string, scid: string): Promise<void> {
  return await invoke("start_scrcpy_server", { id, scid });
}
export {
  getWindows,
  getWindowControls,
  adbDevices,
  reverseServerPort,
  pushServerFile,
  openSocketServer,
  closeSocketServer,
  startScrcpyServer,
};
export type { Device, WindowInfo };
