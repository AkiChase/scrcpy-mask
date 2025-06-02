import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import {
  KeyMapping,
  KeyMappingConfig,
  KeySteeringWheel,
} from "../tools/keyMappingConfig";
import { LocalStore } from "./localStore";
import { setAdbPath } from "../tools/invoke";
import i18n, { allLanguage } from "../i18n";
import { LogicalPosition, LogicalSize } from "@tauri-apps/api/dpi";

export const useGlobalStore = defineStore("global", () => {
  const showLoadingFlag = ref(false);
  async function showLoading() {
    showLoadingFlag.value = true;
  }
  async function hideLoading() {
    showLoadingFlag.value = false;
  }

  interface ControledDevice {
    scid: string;
    deviceName: string;
    deviceID: string;
  }

  const controledDevice: Ref<ControledDevice | null> = ref(null);
  const editKeyMappingList: Ref<KeyMapping[]> = ref([]);

  // Applies the edited key mapping list and checks for duplicate keys
  function applyEditKeyMappingList(): boolean {
    const set = new Set<string>();
    for (const keyMapping of editKeyMappingList.value) {
      if (keyMapping.type === "SteeringWheel") {
        for (const name of ["up", "down", "left", "right"] as const) {
          if (set.has((keyMapping as KeySteeringWheel).key[name])) return false;
          set.add((keyMapping as KeySteeringWheel).key[name]);
        }
      } else if (keyMapping.type !== "Fire") {
        // check duplicated key
        if (set.has(keyMapping.key as string)) return false;
        set.add(keyMapping.key as string);
      }
    }

    keyMappingConfigList.value[curKeyMappingIndex.value].list =
      editKeyMappingList.value;
    LocalStore.set("keyMappingConfigList", keyMappingConfigList.value);
    return true;
  }

  // Reset the edited key mapping to the original key mapping list
  function resetEditKeyMappingList() {
    editKeyMappingList.value = JSON.parse(
      JSON.stringify(keyMappingConfigList.value[curKeyMappingIndex.value].list)
    );
  }

  // change key mapping scheme
  function setCurKeyMappingIndex(index: number) {
    curKeyMappingIndex.value = index;
    resetEditKeyMappingList();
    LocalStore.set("curKeyMappingIndex", index);
  }

  const externalControlled = ref(false);

  const screenSizeW: Ref<number> = ref(0);
  const screenSizeH: Ref<number> = ref(0);

  const curMaskSize = ref({
    w: 0,
    h: 0,
  });
  const curMaskPos = ref({
    x: 0,
    y: 0,
  });

  function setCurMaskSize(size: LogicalSize) {
    curMaskSize.value.w = Math.round(size.width) - 70;
    curMaskSize.value.h = Math.round(size.height) - 30;
  }
  function setCurMaskPos(pos: LogicalPosition) {
    curMaskPos.value.x = Math.round(pos.x) + 70;
    curMaskPos.value.y = Math.round(pos.y) + 30;
  }

  // persistent storage
  const keyMappingConfigList: Ref<KeyMappingConfig[]> = ref([]);
  const curKeyMappingIndex = ref(0);
  const maskKeyTip = ref({
    transparency: 0.5,
    show: true,
  });
  const checkUpdateAtStart = ref(true);

  const screenStream = ref({
    enable: false,
    address: "",
  });

  const rotation = ref({
    enable: true,
    verticalLength: 600,
    horizontalLength: 800,
  });

  const clipboardSync = ref({
    syncFromDevice: true,
    pasteFromPC: true,
  });

  const adbPath = ref("adb");
  function changeAbdPath(path: string) {
    adbPath.value = path;
    setAdbPath(path);
    LocalStore.set("adbPath", path);
  }

  const language = ref<keyof typeof allLanguage>("en-US");
  function setLanguage(lang: keyof typeof allLanguage) {
    if (lang === language.value) return;
    language.value = lang;
    i18n.global.locale = lang;
    LocalStore.set("language", lang);
  }

  return {
    // persistent storage
    keyMappingConfigList,
    curKeyMappingIndex,
    maskKeyTip,
    checkUpdateAtStart,
    externalControlled,
    screenStream,
    rotation,
    clipboardSync,
    adbPath,
    language,
    // in-memory storage
    curMaskSize,
    curMaskPos,
    screenSizeW,
    screenSizeH,
    showLoadingFlag,
    controledDevice,
    editKeyMappingList,
    // action
    showLoading,
    hideLoading,
    setCurMaskPos,
    setCurMaskSize,
    applyEditKeyMappingList,
    resetEditKeyMappingList,
    setCurKeyMappingIndex,
    changeAbdPath,
    setLanguage,
  };
});
