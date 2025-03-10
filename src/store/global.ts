import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import {
  KeyMapping,
  KeyMappingConfig,
  KeySteeringWheel,
} from "../keyMappingConfig";
import { LocalStore } from "./localStore";
import { setAdbPath } from "../invoke";
import i18n, { allLanguage } from "../i18n";

export const useGlobalStore = defineStore("global", () => {
  const showLoadingFlag = ref(false);
  function showLoading() {
    showLoadingFlag.value = true;
  }
  function hideLoading() {
    showLoadingFlag.value = false;
  }

  interface ControledDevice {
    scid: string;
    deviceName: string;
    deviceID: string;
  }

  const controledDevice: Ref<ControledDevice | null> = ref(null);
  const editKeyMappingList: Ref<KeyMapping[]> = ref([]);

  let checkUpdate: () => Promise<void> = async () => {};
  let checkAdb: () => Promise<void> = async () => {};

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

  const keyInputFlag = ref(false);

  const maskSizeW: Ref<number> = ref(0);
  const maskSizeH: Ref<number> = ref(0);

  const screenStreamClientId = ref("scrcpy-mask");

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
    screenStreamClientId, // TODO none reactive
    maskSizeW,
    maskSizeH,
    screenSizeW,
    screenSizeH,
    keyInputFlag, // TODO none reactive
    showLoadingFlag,
    controledDevice,
    editKeyMappingList,
    // action
    showLoading,
    hideLoading,
    applyEditKeyMappingList,
    resetEditKeyMappingList,
    setCurKeyMappingIndex,
    changeAbdPath,
    setLanguage,
    // TODO move to NonReactive Store
    checkUpdate,
    checkAdb,
  };
});
