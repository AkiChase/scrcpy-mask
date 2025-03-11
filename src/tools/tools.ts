import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

export function compareVersion(v1: string, v2: string) {
  const [x1, y1, z1] = v1.split(".");
  const [x2, y2, z2] = v2.split(".");

  if (x1 !== x2) {
    return parseInt(x1) > parseInt(x2) ? 1 : -1;
  }
  if (y1 !== y2) {
    return parseInt(y1) > parseInt(y2) ? 1 : -1;
  }
  if (z1 !== z2) {
    return parseInt(z1) > parseInt(z2) ? 1 : -1;
  }

  return 0;
}

export function genClientId() {
  let result = "";
  const characters =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  const charactersLength = characters.length;
  for (let i = 0; i < 16; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
}

export async function cleanAfterimage() {
  const appWindow = getCurrentWindow();
  const scale = await appWindow.scaleFactor();
  const oldSize = (await appWindow.innerSize()).toLogical(scale);
  const newSize = new LogicalSize(oldSize.width, oldSize.height + 1);
  await appWindow.setSize(newSize);
  setTimeout(() => {
    appWindow.setSize(oldSize);
  }, 150);
}
