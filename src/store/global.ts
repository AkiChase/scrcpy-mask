import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import {
  KeyMapping,
  KeyMappingConfig,
  KeySteeringWheel,
} from "../keyMappingConfig";
import { LocalStore } from "./localStore";

export const useGlobalStore = defineStore("global", () => {
  const showLoadingRef = ref(false);
  function showLoading() {
    showLoadingRef.value = true;
  }
  function hideLoading() {
    showLoadingRef.value = false;
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
  function setKeyMappingIndex(index: number) {
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
  const maskButton = ref({
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

  return {
    // persistent storage
    keyMappingConfigList,
    curKeyMappingIndex,
    maskButton,
    checkUpdateAtStart,
    externalControlled,
    screenStream,
    rotation,
    clipboardSync,
    // in-memory storage
    screenStreamClientId,
    maskSizeW,
    maskSizeH,
    screenSizeW,
    screenSizeH,
    keyInputFlag,
    showLoading,
    hideLoading,
    showLoadingRef,
    controledDevice,
    editKeyMappingList,
    applyEditKeyMappingList,
    resetEditKeyMappingList,
    setKeyMappingIndex,
    checkUpdate,
    checkAdb,
  };
});
