import { emit } from "@tauri-apps/api/event";

enum PureFrontCommandType {
  PasteText = 15,
}

interface PureFrontCommandPayload {
  fcType: PureFrontCommandType;
  receiver?: string;
  msgData: PureFrontCommandData;
}

type PureFrontCommandData = null;

export async function sendPureFrontCommand(
  commandType: PureFrontCommandType,
  msgData: PureFrontCommandData,
  receiver?: string
) {
  const payload: PureFrontCommandPayload = { fcType: commandType, msgData };
  if (receiver !== undefined) payload.receiver = receiver;

  switch (commandType) {
    case PureFrontCommandType.PasteText:
      break;
    default:
      console.error("暂不支持当前命令");
  }
  await emit("front-command", payload);
}
