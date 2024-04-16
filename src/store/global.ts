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
    deviceName: string;
    device: Device;
  }

  const screenSizeW: Ref<number> = ref(0);
  const screenSizeH: Ref<number> = ref(0);

  const controledDevice: Ref<ControledDevice | null> = ref(null);

  return {
    showLoading,
    hideLoading,
    showLoadingRef,
    controledDevice,
    screenSizeW,
    screenSizeH,
  };
});
