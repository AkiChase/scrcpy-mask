import { KeyMappingConfig } from "./keyMappingConfig";
import { useGlobalStore } from "./store/global";
import { useI18n } from "vue-i18n";
import { LocalStore } from "./store/localStore";

let store: ReturnType<typeof useGlobalStore>;
let t: ReturnType<typeof useI18n>["t"];

async function loadKeyMappingConfigList() {
  // loading keyMappingConfigList from local store
  let keyMappingConfigList = await LocalStore.get<KeyMappingConfig[]>(
    "keyMappingConfigList"
  );
  if (keyMappingConfigList === undefined || keyMappingConfigList.length === 0) {
    // add empty key mapping config
    // unable to get mask element when app is not ready
    // so we use the stored mask area to get relative size
    const maskArea = await LocalStore.get<{
      posX: number;
      posY: number;
      sizeW: number;
      sizeH: number;
    }>("maskArea");
    let relativeSize = { w: 800, h: 600 };
    if (maskArea !== undefined) {
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
    await LocalStore.set("keyMappingConfigList", keyMappingConfigList);
  }
  store.keyMappingConfigList = keyMappingConfigList;
}

async function loadCurKeyMappingIndex() {
  // loading curKeyMappingIndex from local store
  let curKeyMappingIndex = await LocalStore.get<number>("curKeyMappingIndex");
  if (
    curKeyMappingIndex === undefined ||
    curKeyMappingIndex >= store.keyMappingConfigList.length
  ) {
    curKeyMappingIndex = 0;
    LocalStore.set("curKeyMappingIndex", curKeyMappingIndex);
  }
  store.curKeyMappingIndex = curKeyMappingIndex;
}

async function loadMaskButton() {
  // loading maskButton from local store
  const maskButton = await LocalStore.get<{
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
  const checkUpdateAtStart = await LocalStore.get<boolean>(
    "checkUpdateAtStart"
  );
  store.checkUpdateAtStart = checkUpdateAtStart ?? true;
}

async function loadRotation() {
  // loading rotation from local store
  const rotation = await LocalStore.get<{
    enable: boolean;
    verticalLength: number;
    horizontalLength: number;
  }>("rotation");
  if (rotation) store.rotation = rotation;
}

async function loadScreenStream() {
  // loading screenStream from local store
  const screenStream = await LocalStore.get<{
    enable: boolean;
    address: string;
  }>("screenStream");
  if (screenStream) store.screenStream = screenStream;
}

async function loadClipboardSync() {
  // loading clipboardSync from local store
  const clipboardSync = await LocalStore.get<{
    syncFromDevice: boolean;
    pasteFromPC: boolean;
  }>("clipboardSync");
  if (clipboardSync) store.clipboardSync = clipboardSync;
}

export async function loadPersistentStorage(
  theStore: ReturnType<typeof useGlobalStore>,
  theT: ReturnType<typeof useI18n>["t"]
) {
  await LocalStore.init();

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
