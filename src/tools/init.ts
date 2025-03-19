import { LocalStore } from "../store/localStore";
import { NonReactiveStore } from "../store/noneReactiveStore";
import { useGlobalStore } from "../store/global";
import { useCheckAdb, useCheckUpdate } from "./hooks";
import { genClientId } from "./tools";
import { getCurrentWindow } from "@tauri-apps/api/window";

export async function primaryInit() {
  await LocalStore.init();
}

let unlistenResize = () => {};
let unlistenMove = () => {};
export async function secondaryInit() {
  const store = useGlobalStore();
  // init screenStreamClientId
  NonReactiveStore.mem.screenStreamClientId = genClientId();
  // check update
  if (store.checkUpdateAtStart) useCheckUpdate()();
  // check adb available
  useCheckAdb()();

  // listen to window event
  const appWindow = getCurrentWindow();
  const scaleFactor = await appWindow.scaleFactor();
  unlistenResize = await appWindow.onResized(({ payload: size }) => {
    store.setCurMaskSize(size.toLogical(scaleFactor));
  });
  unlistenMove = await appWindow.onMoved(({ payload: position }) => {
    store.setCurMaskPos(position.toLogical(scaleFactor));
  });
  store.setCurMaskSize((await appWindow.innerSize()).toLogical(scaleFactor));
  store.setCurMaskPos((await appWindow.innerPosition()).toLogical(scaleFactor));
}

export function secondaryClean() {
  unlistenResize?.();
  unlistenMove?.();
}
