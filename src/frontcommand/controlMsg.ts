import { emit } from "@tauri-apps/api/event";
import {
  AndroidKeyEventAction,
  AndroidKeycode,
  AndroidMetastate,
  AndroidMotionEventAction,
  AndroidMotionEventButtons,
} from "./android";

interface ControlMsgPayload {
  msgType: ControlMsgType;
  msgData?: ControlMsgData;
}

async function sendControlMsg(payload: ControlMsgPayload) {
  await emit("front-command", payload);
}

export async function sendInjectKeycode(payload: InjectKeycode) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeInjectKeycode,
    msgData: payload,
  });
}

export async function sendInjectText(payload: InjectText) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeInjectText,
    msgData: payload,
  });
}

export async function sendInjectTouchEvent(payload: InjectTouchEvent) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeInjectTouchEvent,
    msgData: payload,
  });
}

export async function sendInjectScrollEvent(payload: InjectScrollEvent) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeInjectScrollEvent,
    msgData: payload,
  });
}

export async function sendBackOrScreenOn(payload: BackOrScreenOn) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeBackOrScreenOn,
    msgData: payload,
  });
}

export async function sendExpandNotificationPanel() {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeExpandNotificationPanel,
  });
}

export async function sendExpandSettingsPanel() {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeExpandSettingsPanel,
  });
}

export async function sendCollapsePanels() {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeCollapsePanels,
  });
}

export async function sendGetClipboard(payload: GetClipboard) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeGetClipboard,
    msgData: payload,
  });
}

export async function sendSetClipboard(payload: SetClipboard) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeSetClipboard,
    msgData: payload,
  });
}

export async function sendSetScreenPowerMode(payload: SetScreenPowerMode) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeSetScreenPowerMode,
    msgData: payload,
  });
}

export async function sendRotateDevice() {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeRotateDevice,
  });
}

export async function sendUhidCreate(payload: UhidCreate) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeUhidCreate,
    msgData: payload,
  });
}

export async function sendUhidInput(payload: UhidInput) {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeUhidInput,
    msgData: payload,
  });
}

export async function sendOpenHardKeyboardSettings() {
  await sendControlMsg({
    msgType: ControlMsgType.ControlMsgTypeOpenHardKeyboardSettings,
  });
}

export enum ControlMsgType {
  ControlMsgTypeInjectKeycode, //发送原始按键
  ControlMsgTypeInjectText, //发送文本，不知道是否能输入中文（估计只是把文本转为keycode的输入效果）
  ControlMsgTypeInjectTouchEvent, //发送触摸事件
  ControlMsgTypeInjectScrollEvent, //发送滚动事件（类似接入鼠标后滚动滚轮的效果，不是通过触摸实现的）
  ControlMsgTypeBackOrScreenOn, //应该就是发送返回键
  ControlMsgTypeExpandNotificationPanel, //打开消息面板
  ControlMsgTypeExpandSettingsPanel, //打开设置面板（就是消息面板右侧的）
  ControlMsgTypeCollapsePanels, //折叠上述面板
  ControlMsgTypeGetClipboard, //获取剪切板
  ControlMsgTypeSetClipboard, //设置剪切板
  ControlMsgTypeSetScreenPowerMode, //设置屏幕电源模式，是关闭设备屏幕的（SC_SCREEN_POWER_MODE_OFF 和 SC_SCREEN_POWER_MODE_NORMAL ）
  ControlMsgTypeRotateDevice, //旋转设备屏幕
  ControlMsgTypeUhidCreate, //创建虚拟设备？从而模拟真实的键盘、鼠标用的，目前没用
  ControlMsgTypeUhidInput, //同上转发键盘、鼠标的输入，目前没用
  ControlMsgTypeOpenHardKeyboardSettings, //打开设备的硬件键盘设置，目前没用
}

type ControlMsgData =
  | InjectKeycode
  | InjectText
  | InjectTouchEvent
  | InjectScrollEvent
  | BackOrScreenOn
  | GetClipboard
  | SetClipboard
  | SetScreenPowerMode
  | UhidCreate
  | UhidInput;

interface ScPosition {
  x: number;
  y: number;
  // screen width
  w: number;
  // screen height
  h: number;
}

interface InjectKeycode {
  action: AndroidKeyEventAction;
  keycode: AndroidKeycode;
  // https://developer.android.com/reference/android/view/KeyEvent#getRepeatCount()
  repeat: number;
  metastate: AndroidMetastate;
}

export enum ScCopyKey {
  SC_COPY_KEY_NONE,
  SC_COPY_KEY_COPY,
  SC_COPY_KEY_CUT,
}

export enum ScScreenPowerMode {
  // see <https://android.googlesource.com/platform/frameworks/base.git/+/pie-release-2/core/java/android/view/SurfaceControl.java#305>
  SC_SCREEN_POWER_MODE_OFF = 0,
  SC_SCREEN_POWER_MODE_NORMAL = 2,
}

interface InjectText {
  text: string;
}

interface InjectTouchEvent {
  action: AndroidMotionEventAction;
  actionButton: AndroidMotionEventButtons;
  buttons: AndroidMotionEventButtons;
  pointerId: number;
  position: ScPosition;
  pressure: number;
}

interface InjectScrollEvent {
  position: ScPosition;
  hscroll: number;
  vscroll: number;
  buttons: AndroidMotionEventButtons;
}

interface BackOrScreenOn {
  action: AndroidKeyEventAction; // action for the BACK key
  // screen may only be turned on on ACTION_DOWN
}

interface GetClipboard {
  copyKey: ScCopyKey;
}

interface SetClipboard {
  sequence: number;
  text: string;
  paste: boolean;
}

interface SetScreenPowerMode {
  mode: ScScreenPowerMode;
}

interface UhidCreate {
  id: number;
  reportDescSize: number;
  reportDesc: Uint8Array;
}

interface UhidInput {
  id: number;
  size: number;
  data: Uint8Array;
}
