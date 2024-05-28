import { Store } from "@tauri-apps/plugin-store";
import { KeyMappingConfig } from "./keyMappingConfig";
import { useGlobalStore } from "./store/global";
import { useI18n } from "vue-i18n";

let localStore: Store;
let store: ReturnType<typeof useGlobalStore>;
let t: ReturnType<typeof useI18n>["t"];

async function loadKeyMappingConfigList() {
  // loading keyMappingConfigList from local store
  let keyMappingConfigList = await localStore.get<KeyMappingConfig[]>(
    "keyMappingConfigList"
  );
  if (keyMappingConfigList === null || keyMappingConfigList.length === 0) {
    // add empty key mapping config
    // unable to get mask element when app is not ready
    // so we use the stored mask area to get relative size
    const maskArea = await localStore.get<{
      posX: number;
      posY: number;
      sizeW: number;
      sizeH: number;
    }>("maskArea");
    let relativeSize = { w: 800, h: 600 };
    if (maskArea !== null) {
      relativeSize = {
        w: maskArea.sizeW,
        h: maskArea.sizeH,
      };
    }
    keyMappingConfigList = [
      {
        relativeSize,
        title: t("pages.Mask.blankConfig"),
        list: [],
      },
    ];
    await localStore.set("keyMappingConfigList", keyMappingConfigList);
  }
  store.keyMappingConfigList = keyMappingConfigList;
}

async function loadCurKeyMappingIndex() {
  // loading curKeyMappingIndex from local store
  let curKeyMappingIndex = await localStore.get<number>("curKeyMappingIndex");
  if (
    curKeyMappingIndex === null ||
    curKeyMappingIndex >= store.keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    localStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  store.curKeyMappingIndex = curKeyMappingIndex;
}

async function loadMaskButton() {
  // loading maskButton from local store
  let maskButton = await localStore.get<{
    show: boolean;
    transparency: number;
  }>("maskButton");
  store.maskButton = maskButton ?? {
    show: true,
    transparency: 0.5,
  };
}

async function loadCheckUpdateAtStart() {
  // loading checkUpdateAtStart from local store
  const checkUpdateAtStart = await localStore.get<boolean>(
    "checkUpdateAtStart"
  );
  store.checkUpdateAtStart = checkUpdateAtStart ?? true;
}

async function loadRotation() {
  // loading rotation from local store
  const rotation = await localStore.get<{
    enable: boolean;
    verticalLength: number;
    horizontalLength: number;
  }>("rotation");
  if (rotation) store.rotation = rotation;
}

async function loadScreenStream() {
  // loading screenStream from local store
  const screenStream = await localStore.get<{
    enable: boolean;
    address: string;
  }>("screenStream");
  if (screenStream) store.screenStream = screenStream;
}

async function loadClipboardSync() {
  // loading clipboardSync from local store
  const clipboardSync = await localStore.get<{
    syncFromDevice: boolean;
    pasteFromPC: boolean;
  }>("clipboardSync");
  if (clipboardSync) store.clipboardSync = clipboardSync;
  console.log(store.clipboardSync);
}

export async function loadLocalStorage(
  theLocalStore: Store,
  theStore: ReturnType<typeof useGlobalStore>,
  theT: ReturnType<typeof useI18n>["t"]
) {
  localStore = theLocalStore;
  store = theStore;
  t = theT;

  await loadKeyMappingConfigList();
  await loadCurKeyMappingIndex();
  await loadMaskButton();
  await loadCheckUpdateAtStart();
  await loadRotation();
  await loadScreenStream();
  await loadClipboardSync();
}
