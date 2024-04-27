import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import { Device } from "../invoke";
import { KeyMapping, KeyMappingConfig } from "../keyMappingConfig";
import { Store } from "@tauri-apps/plugin-store";

const localStore = new Store("store.bin");

export const useGlobalStore = defineStore("counter", () => {
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

  function applyEditKeyMappingList() {
    keyMappingConfigList.value[curKeyMappingIndex.value].list =
      editKeyMappingList.value;
    localStore.set("keyMappingConfigList", keyMappingConfigList.value);
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
    applyEditKeyMappingList,
    resetEditKeyMappingList,
    setKeyMappingIndex,
  };
});
