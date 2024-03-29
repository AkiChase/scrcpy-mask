import { defineStore } from "pinia";
import { Ref, ref } from "vue";
import { Device } from "../invoke";

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
    smid: string;
    deviceName: string;
    device: Device;
  }

  const controledDevices: Ref<ControledDevice[]> = ref([]);

  return {
    showLoading,
    hideLoading,
    showLoadingRef,
    controledDevices,
  };
});
