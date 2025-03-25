import { useMessage } from "naive-ui";
import { useGlobalStore } from "../store/global";
import { sendKey, shutdown, swipe, touch } from "../frontcommand/scrcpyMaskCmd";
import { useI18n } from "vue-i18n";
import { error } from "@tauri-apps/plugin-log";

let ws: WebSocket;
let sharedMessage: ReturnType<typeof useMessage>;
let sharedStore: ReturnType<typeof useGlobalStore>;
let t: ReturnType<typeof useI18n>["t"];

export function connectExternalControl(
  url: string,
  message: ReturnType<typeof useMessage>,
  store: ReturnType<typeof useGlobalStore>,
  i18nT: ReturnType<typeof useI18n>["t"]
) {
  sharedMessage = message;
  sharedStore = store;
  t = i18nT;

  ws = new WebSocket(url);
  ws.addEventListener("open", handleOpen);
  ws.addEventListener("message", handleMessage);
  ws.addEventListener("close", handleClose);
  ws.addEventListener("error", handleError);
}

export function closeExternalControl() {
  if (ws) ws.close();
}

function handleOpen() {
  sharedStore.externalControlled = true;
  sharedStore.hideLoading();
  sharedMessage.success(t("websocket.open"));
}

async function handleMessage(event: MessageEvent) {
  try {
    const msg = JSON.parse(event.data);
    if (msg.type === "showMessage") {
      sharedMessage.create(msg.msgContent, { type: msg.msgType });
    } else if (msg.type === "getControlledDevice") {
      msg.controledDevice = sharedStore.controledDevice;
      ws.send(JSON.stringify(msg));
    } else if (msg.type === "sendKey") {
      delete msg.type;
      await sendKey(msg);
    } else if (msg.type === "touch") {
      msg.screen = { w: sharedStore.screenSizeW, h: sharedStore.screenSizeH };
      delete msg.type;
      await touch(msg);
    } else if (msg.type === "swipe") {
      msg.screen = { w: sharedStore.screenSizeW, h: sharedStore.screenSizeH };
      delete msg.type;
      await swipe(msg);
    } else if (msg.type === "shutdown") {
      await shutdown();
      sharedStore.controledDevice = null;
    } else {
      error("Invalid message received: " + msg);
    }
  } catch (e) {
    error("Message received failed, " + e);
    console.error(e);
  }
}

function handleClose() {
  sharedMessage.info(t("websocket.close"));
  ws.close();
  sharedStore.externalControlled = false;
  sharedStore.hideLoading();
}

function handleError() {
  sharedMessage.error(t("websocket.error"));
  ws.close();
  sharedStore.externalControlled = false;
  sharedStore.hideLoading();
}
