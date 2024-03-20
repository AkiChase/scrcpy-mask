import { emit } from "@tauri-apps/api/event";
import { AndroidKeycode, AndroidMetastate } from "./android";

async function sendScrcpyMaskCmd(
  commandType: ScrcpyMaskCmdType,
  msgData: ScrcpyMaskCmdData
) {
  const payload: ScrcpyMaskCmdPayload = { msgType: commandType, msgData };
  await emit("front-command", payload);
}

export async function sendKey(payload: CmdDataSendKey) {
  await sendScrcpyMaskCmd(ScrcpyMaskCmdType.SendKey, payload);
}

export async function touch(payload: CmdDataTouch) {
  await sendScrcpyMaskCmd(ScrcpyMaskCmdType.Touch, payload);
}

export async function swipe(payload: CmdDataSwipe) {
  await sendScrcpyMaskCmd(ScrcpyMaskCmdType.Swipe, payload);
}

export async function shutdown(payload: CmdDataShutdown) {
  await sendScrcpyMaskCmd(ScrcpyMaskCmdType.Shutdown, payload);
}

export enum ScrcpyMaskCmdType {
  SendKey = 15,
  Touch = 16,
  Swipe = 17,
  Shutdown = 18,
}

type ScrcpyMaskCmdData =
  | CmdDataSendKey
  | CmdDataTouch
  | CmdDataSwipe
  | CmdDataShutdown;

enum SendKeyAction {
  Default = 0,
  Down = 1,
  Up = 2,
}

interface CmdDataSendKey {
  action: SendKeyAction;
  keycode: AndroidKeycode;
  metastate?: AndroidMetastate;
}

export enum TouchAction {
  Default = 0,
  Down = 1,
  Up = 2,
}

interface CmdDataTouch {
  action: TouchAction;
  pointerId: number;
  screen: { w: number; h: number };
  pos: { x: number; y: number };
}

export enum SwipeAction {
  Default = 0,
  // cooperate with touch action
  NoUp = 1,
  NoDown = 2,
}

interface CmdDataSwipe {
  action: SwipeAction;
  pointerId: number;
  screen: { w: number; h: number };
  pos: { x: number; y: number }[];
  intervalBetweenPos: number;
}

interface CmdDataShutdown {
  scId: string;
}

interface ScrcpyMaskCmdPayload {
  msgType: ScrcpyMaskCmdType;
  msgData: ScrcpyMaskCmdData;
}
