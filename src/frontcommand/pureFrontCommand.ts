import { emit } from "@tauri-apps/api/event";

enum ScrcpyMaskCmdType {
  PasteText = 15,
}

interface ScrcpyMaskCmdPayload {
  msgType: ScrcpyMaskCmdType;
  receiver?: string;
  msgData: ScrcpyMaskCmdData;
}

type ScrcpyMaskCmdData = null;

export async function sendScrcpyMaskCmd(
  commandType: ScrcpyMaskCmdType,
  msgData: ScrcpyMaskCmdData,
  receiver?: string
) {
  const payload: ScrcpyMaskCmdPayload = { msgType: commandType, msgData };
  if (receiver !== undefined) payload.receiver = receiver;

  switch (commandType) {
    case ScrcpyMaskCmdType.PasteText:
      break;
    default:
      console.error("暂不支持当前命令");
  }
  await emit("front-command", payload);
}
