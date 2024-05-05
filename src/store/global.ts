import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import { Device } from "../invoke";
import {
  KeyMapping,
  KeyMappingConfig,
  KeySteeringWheel,
} from "../keyMappingConfig";
import { Store } from "@tauri-apps/plugin-store";

const localStore = new Store("store.bin");

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
    device: Device;
  }

  const screenSizeW: Ref<number> = ref(0);
  const screenSizeH: Ref<number> = ref(0);

  const controledDevice: Ref<ControledDevice | null> = ref(null);

  const keyMappingConfigList: Ref<KeyMappingConfig[]> = ref([]);
  const curKeyMappingIndex = ref(0);
  const editKeyMappingList: Ref<KeyMapping[]> = ref([]);

  const maskButton = ref({
    transparency: 0.5,
    show: true,
  });

  function applyEditKeyMappingList(): boolean {
    const set = new Set<string>();
    for (const keyMapping of editKeyMappingList.value) {
      if (keyMapping.type === "SteeringWheel") {
        const nameList: ["up", "down", "left", "right"] = [
          "up",
          "down",
          "left",
          "right",
        ];
        for (const name of nameList) {
          if (set.has((keyMapping as KeySteeringWheel).key[name])) return false;
          set.add((keyMapping as KeySteeringWheel).key[name]);
        }
      } else {
        if (set.has(keyMapping.key as string)) return false;
        set.add(keyMapping.key as string);
      }
    }

    keyMappingConfigList.value[curKeyMappingIndex.value].list =
      editKeyMappingList.value;
    localStore.set("keyMappingConfigList", keyMappingConfigList.value);
    return true;
  }

  function resetEditKeyMappingList() {
    editKeyMappingList.value = JSON.parse(
      JSON.stringify(keyMappingConfigList.value[curKeyMappingIndex.value].list)
    );
  }

  function setKeyMappingIndex(index: number) {
    curKeyMappingIndex.value = index;
    resetEditKeyMappingList();
    localStore.set("curKeyMappingIndex", index);
  }

  return {
    showLoading,
    hideLoading,
    showLoadingRef,
    controledDevice,
    screenSizeW,
    screenSizeH,
    keyMappingConfigList,
    curKeyMappingIndex,
    editKeyMappingList,
    maskButton,
    applyEditKeyMappingList,
    resetEditKeyMappingList,
    setKeyMappingIndex,
  };
});
